use std::net::SocketAddr;

use color_eyre::eyre::Context;
use color_eyre::{Result, Section};
use tokio::io::{AsyncBufReadExt, BufStream};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Sender;

use super::Event;
use tracing::{info, warn};

pub async fn handle_data_sources(addr: SocketAddr, share: &Sender<Event>) -> Result<()> {
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
