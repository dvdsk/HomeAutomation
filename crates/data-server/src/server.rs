use color_eyre::eyre::Context;
use std::mem;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufStream};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, Sender};
use tokio::time::timeout;

use color_eyre::{Result, Section};
use protocol::{Reading, SensorMessage};

use tracing::{info, warn};

pub async fn handle_data_source(stream: TcpStream, queue: Sender<Event>) {
    let mut reader = BufStream::new(stream);
    let mut buf = Vec::new();
    loop {
        buf.clear();
        let n_read = match reader.read_until(0, &mut buf).await {
            Err(e) => {
                warn!("Connection failed/closed: {e}");
                return;
            }
            Ok(bytes) => bytes,
        };

        let bytes = &mut buf[0..n_read];
        if bytes.is_empty() {
            //eof
            warn!("end of stream");
            return;
        }

        let decoded = match protocol::Msg::<50>::decode(bytes) {
            Ok(msg) => msg,
            Err(e) => {
                warn!("decode failed: {e:?}");
                return;
            }
        };
        match decoded {
            protocol::Msg::Readings(readings) => {
                for value in readings.values {
                    queue
                        .send(Event::NewReading(Ok(value)))
                        .await
                        .expect("fn spread_updates should stay running");
                }
            }
            protocol::Msg::ErrorReport(report) => queue
                .send(Event::NewReading(Err(Box::new(report.error))))
                .await
                .expect("fn spread_updates should stay running"),
        }
    }
}

pub enum Event {
    NewSub(TcpStream),
    NewReading(Result<Reading, Box<protocol::Error>>),
}

pub async fn handle_sub_conn(mut sub: TcpStream, mut tx: mpsc::Receiver<Arc<[u8]>>) {
    loop {
        let msg = tx.recv().await.expect("updates always keep coming");
        if let Err(e) = timeout(Duration::from_secs(2), sub.write_all(&msg)).await {
            warn!("Subscriber connection failed: {e}");
            break;
        }
    }
}

pub async fn spread_updates(mut events: mpsc::Receiver<Event>) -> Result<()> {
    let mut subscribers = Vec::new();

    loop {
        let event = events
            .recv()
            .await
            .expect("queue is kept open by register_subs");

        let msg = match event {
            Event::NewSub(sub) => {
                let (tx, rx) = tokio::sync::mpsc::channel(100);
                tokio::spawn(handle_sub_conn(sub, rx));
                subscribers.push(tx);
                continue;
            }
            Event::NewReading(Ok(reading)) => {
                // PERF:
                // use futures-util's peekable with next_if
                // to get up to 49 extra messages for efficiency
                let mut readings: SensorMessage<50> = SensorMessage::default();
                readings
                    .values
                    .push(reading)
                    .expect("capacity should be > 0");
                protocol::Msg::Readings(readings)
            }
            Event::NewReading(Err(err)) => {
                let report = protocol::ErrorReport::new(*err);
                protocol::Msg::ErrorReport(report)
            }
        };

        let bytes = msg.encode();
        let bytes: Arc<[u8]> = Arc::from(bytes.into_boxed_slice());

        let subs = mem::take(&mut subscribers);
        for sub in subs {
            if sub.try_send(bytes.clone()).is_ok() {
                subscribers.push(sub);
            }
        }
    }
}

pub async fn handle_data_sources(addr: SocketAddr, share: &mpsc::Sender<Event>) -> Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .wrap_err("Could not start listening for new subscribers")
        .with_note(|| format!("trying to listen on: {addr}"))?;

    loop {
        let res = listener.accept().await;
        match res {
            Ok((stream, source)) => {
                info!("new data source connected from: {source}");
                tokio::spawn(handle_data_source(stream, share.clone()));
            }
            Err(e) => {
                println!("new connection failed: {e}");
                continue;
            }
        };
    }
}

pub async fn register_subs(addr: SocketAddr, tx: &mpsc::Sender<Event>) -> Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .wrap_err("Could not start receiving updates")
        .with_note(|| format!("trying to listen on: {addr}"))?;

    loop {
        let res = listener.accept().await;
        match res {
            Ok((mut stream, source)) => {
                match check_sub(&mut stream).await {
                    Ok(name) => info!("new subscriber '{name}' connected from: {source}"),
                    Err(e) => {
                        warn!(
                            "newly connected subscriber from: {source} failed to\
                            pass check: {e}, disconnecting"
                        );
                        continue;
                    }
                }
                tx.send(Event::NewSub(stream))
                    .await
                    .expect("spread_updates (rx) never ends");
            }
            Err(e) => {
                warn!("new connection failed: {e}");
                continue;
            }
        };
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CheckErr {
    #[error("Could not read name length: {0}")]
    ReadLen(std::io::Error),
    #[error("Timed out reading subscriber name")]
    Timeout,
    #[error("Name could not be parsed: {0}")]
    ParseName(std::string::FromUtf8Error),
}

async fn check_sub(stream: &mut TcpStream) -> Result<String, CheckErr> {
    let name_len = timeout(Duration::from_millis(200), stream.read_u8())
        .await
        .map_err(|_| CheckErr::Timeout)?
        .map_err(CheckErr::ReadLen)?;
    let mut buf = vec![0; name_len as usize];
    timeout(Duration::from_millis(200), stream.read_exact(&mut buf))
        .await
        .map_err(|_| CheckErr::Timeout)?
        .map_err(CheckErr::ReadLen)?;

    let name = String::from_utf8(buf).map_err(CheckErr::ParseName)?;
    Ok(name)
}
