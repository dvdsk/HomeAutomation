use std::net::SocketAddr;

use color_eyre::eyre::{eyre, Context};
use color_eyre::{Result, Section};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Sender};

use super::Event;
use tracing::{info, warn};

use super::affector::{track_and_control_affectors, Registar};

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
                tokio::spawn(handle_node(stream, share.clone(), registar.clone()));
            }
            Err(e) => {
                println!("new connection failed: {e}");
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

async fn handle_node(stream: TcpStream, queue: Sender<Event>, registar: Registar) {
    let (reader, writer) = stream.into_split();
    let reader = BufReader::new(reader);

    let (tx, rx) = mpsc::channel(100);
    tokio::join!(
        receive_and_spread_updates(reader, queue, tx),
        track_and_control_affectors(writer, rx, registar),
    );
}

async fn receive_and_spread_updates(
    mut reader: BufReader<OwnedReadHalf>,
    queue: Sender<Event>,
    affectors: Sender<protocol::Affector>,
) {
    let mut buf = Vec::new();

    loop {
        let msg = match read_and_decode_packet(&mut reader, &mut buf).await {
            Ok(decoded) => decoded,
            Err(e) => {
                warn!("Error while reading and decoding packet: {e}");
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
            protocol::Msg::AffectorList(list) => {
                for affector_state in list.values {
                    affectors
                        .send(affector_state)
                        .await
                        .expect("fn track_and_control_affectors should not end")
                }
            }
        }
    } // loop
}
