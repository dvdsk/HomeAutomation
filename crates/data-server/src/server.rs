use color_eyre::eyre::Context;
use std::mem;
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufStream};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, Sender};

use color_eyre::{Result, Section};
use protocol::{Reading, SensorMessage};

use tracing::{info, warn};

pub async fn handle_client(stream: TcpStream, tx: Sender<Event>) {
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
        if bytes.is_empty() { //eof
            warn!("end of stream");
            return;
        }

        tracing::trace!("{:?}", &bytes);
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
                    tx.send(Event::NewReading(Ok(value))).await.unwrap();
                }
            }
            protocol::Msg::ErrorReport(report) => {
                tx.send(Event::NewReading(Err(report.error))).await.unwrap()
            }
        }
    }
}

pub enum Event {
    NewSub(TcpStream),
    NewReading(Result<Reading, protocol::Error>),
}

pub async fn spread_updates(mut events: mpsc::Receiver<Event>) -> Result<()> {
    let mut subscribers = Vec::new();

    while let Some(event) = events.recv().await {
        let msg = match event {
            Event::NewSub(sub) => {
                subscribers.push(sub);
                continue;
            }
            Event::NewReading(Ok(reading)) => {
                // TODO use futures-util's peekable with next_if
                // to get up to 49 extra messages for efficiency
                let mut readings: SensorMessage<50> = SensorMessage::new();
                readings
                    .values
                    .push(reading)
                    .expect("capacity should be > 0");
                protocol::Msg::Readings(readings)
            }
            Event::NewReading(Err(err)) => {
                let report = protocol::ErrorReport::new(err);
                protocol::Msg::ErrorReport(report)
            }
        };

        let bytes = msg.encode();
        let subs = mem::take(&mut subscribers);
        for mut sub in subs.into_iter() {
            if let Err(e) = sub.write_all(&bytes).await {
                warn!("Error writing to subscriber: {e}");
            } else {
                subscribers.push(sub);
            }
        }
    }

    Ok(())
}

pub async fn handle_data_sources(port: u16, share: &mpsc::Sender<Event>) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr)
        .await
        .wrap_err("Could not start listening for new subscribers")
        .with_note(|| format!("trying to listen on port: {port}"))?;

    loop {
        let res = listener.accept().await;
        match res {
            Ok((stream, source)) => {
                info!("new data source connected from: {source}");
                tokio::spawn(handle_client(stream, share.clone()));
            }
            Err(e) => {
                println!("new connection failed: {e}");
                continue;
            }
        };
    }
}

pub async fn register_subs(port: u16, tx: &mpsc::Sender<Event>) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr)
        .await
        .wrap_err("Could not start receiving updates")
        .with_note(|| format!("trying to listen on port: {port}"))?;

    loop {
        let res = listener.accept().await;
        match res {
            Ok((stream, source)) => {
                info!("new subscriber connected from: {source}");
                tx.send(Event::NewSub(stream)).await.unwrap();
            }
            Err(e) => {
                warn!("new connection failed: {e}");
                continue;
            }
        };
    }
}
