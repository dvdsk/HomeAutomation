use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::time::Duration;

use color_eyre::eyre::{eyre, Context};
use color_eyre::{Result, Section};
use futures_concurrency::future::Race;
use governor::clock::Clock;
use governor::{Quota, RateLimiter};
use protocol::Affector;
use socket2::TcpKeepalive;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;

use super::Event;
use tracing::{error, info, instrument, warn};

use super::affector::{control_affectors, Registar};

/// spawns tasks for all nodes that connect. Those tasks push the readings
/// their nodes report into share and send relevant affector orders to the
/// node if they get them via the registar.
pub async fn handle_nodes(
    addr: SocketAddr,
    share: &Sender<Event>,
    registar: Registar,
) -> Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .wrap_err("Could not start listening for new subscribers")
        .with_note(|| format!("trying to listen on: {addr}"))?;

    loop {
        let res = listener.accept().await;
        match res {
            Ok((stream, source)) => {
                info!("new data source connected from: {source}");
                tokio::spawn(handle_node(stream, source, share.clone(), registar.clone()));
            }
            Err(e) => {
                warn!("new connection failed: {e}");
                continue;
            }
        };
    }
}

pub async fn read_and_decode_packet(
    reader: &mut BufReader<OwnedReadHalf>,
    buf: &mut Vec<u8>,
) -> color_eyre::Result<protocol::Msg<50>> {
    buf.clear();
    let n_read = reader
        .read_until(0, buf)
        .await
        .wrap_err("Connection failed/closed")?;

    let bytes = &mut buf[0..n_read];
    if bytes.is_empty() {
        return Err(eyre!("End of stream, connection is closed"));
    }

    protocol::Msg::<50>::decode(bytes).wrap_err("decode failed")
}

#[instrument(skip(stream, queue, registar))]
async fn handle_node(
    stream: TcpStream,
    source: SocketAddr,
    queue: Sender<Event>,
    registar: Registar,
) {
    use tracing_futures::Instrument;

    let sock_ref = socket2::SockRef::from(&stream);
    sock_ref
        .set_tcp_keepalive(
            &TcpKeepalive::new()
                .with_time(Duration::from_secs(5))
                .with_retries(1) // 1 is minimum
                .with_interval(Duration::from_secs(1)),
        )
        .expect("4 sec keepalive should work on tcp stream");
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let affectors = match handshake(&mut reader).await {
        Ok(affectors) => affectors,
        Err(e) => {
            error!("failed node handshake: {e}");
            return;
        }
    };
    info!("new node connected with affectors: {affectors:?}");

    let (tx, rx) = tokio::sync::mpsc::channel(10);
    let key = registar.register(tx, affectors);
    (
        receive_and_spread_updates(reader, queue).in_current_span(),
        control_affectors(writer, rx).in_current_span(),
    )
        .race()
        .await;

    registar.remove(key);
    warn!("node removed (lost connection)");
}

async fn handshake(reader: &mut BufReader<OwnedReadHalf>) -> Result<Vec<Affector>, String> {
    let mut buf = Vec::new();
    let msg = match read_and_decode_packet(reader, &mut buf).await {
        Ok(decoded) => decoded,
        Err(e) => {
            return Err(format!("Error while reading and decoding packet: {e}"));
        }
    };
    let protocol::Msg::AffectorList(list) = msg else {
        return Err("Must get affector list as first message (handshake)".to_owned());
    };

    Ok(list.values.to_vec())
}

#[instrument(skip_all)]
async fn receive_and_spread_updates(mut reader: BufReader<OwnedReadHalf>, queue: Sender<Event>) {
    let quota = Quota::per_second(NonZeroU32::new(40).unwrap())
        .allow_burst(NonZeroU32::new(200u32).unwrap());
    let limiter = RateLimiter::direct(quota);

    let mut buf = Vec::new();
    loop {
        if let Err(allowed_again) = limiter.check() {
            let now = governor::clock::DefaultClock::default().now();
            let allowed_in = allowed_again.wait_time_from(now);
            error!(
                "Refusing to read packet: (generous) ratelimit surpassed. \
                Next packet allowed in {allowed_in:?}. Sleeping till then"
            );
            tokio::time::sleep(allowed_in).await;
        }

        let msg = match read_and_decode_packet(&mut reader, &mut buf).await {
            Ok(decoded) => decoded,
            Err(e) => {
                error!("Error while reading and decoding packet: {e}");
                return;
            }
        };

        match msg {
            protocol::Msg::Readings(list) => {
                for value in list.values {
                    queue
                        .send(Event::NewReading(Ok(value)))
                        .await
                        .expect("fn spread_updates should stay running");
                }
            }
            protocol::Msg::ErrorReport(report) => {
                let boxed = Box::new(report.error);
                queue
                    .send(Event::NewReading(Err(boxed)))
                    .await
                    .expect("fn spread_updates should stay running");
            }
            protocol::Msg::AffectorList(_) => {
                error!("Affector list should only be send at the start of the connection");
                return;
            }
        }
    } // Loop
}
