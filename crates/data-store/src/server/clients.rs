use std::net::SocketAddr;

use color_eyre::eyre::Context;
use color_eyre::Section;
use futures::{SinkExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_serde::formats::Bincode;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{debug, error, info, warn};

use super::db::{Data, Logs, Stats};
use crate::api::{self, ServerError};

type Conn = tokio_serde::Framed<
    Framed<TcpStream, LengthDelimitedCodec>,
    api::Request,
    api::Response,
    Bincode<api::Request, api::Response>,
>;

pub(crate) async fn handle(
    port: u16,
    data: Data,
    stats: Stats,
    logs: Logs,
) -> color_eyre::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .wrap_err("Could not bind to address")
        .with_note(|| format!("port: {port}"))?;

    loop {
        let (socket, source) = match listener.accept().await {
            Err(e) => {
                warn!("client could not connect: {e}");
                continue;
            }
            Ok(res) => res,
        };

        let Some(conn) = handshake_and_log(socket, source).await else {
            continue;
        };

        tokio::task::spawn(handle_client(
            conn,
            data.clone(),
            stats.clone(),
            logs.clone(),
        ));
    }
}

async fn handshake_and_log(stream: TcpStream, source: SocketAddr) -> Option<Conn> {
    let length_delimited = Framed::new(
        stream,
        LengthDelimitedCodec::builder()
            .max_frame_length(api::MAX_PACKAGE_SIZE)
            .new_codec(),
    );
    let mut stream: tokio_serde::Framed<_, api::Request, api::Response, _> =
        tokio_serde::Framed::new(length_delimited, Bincode::default());

    match stream.try_next().await {
        Ok(Some(api::Request::Handshake { name })) => {
            info!("Client {name} connected from {source}");
            return Some(stream);
        }
        Ok(Some(other)) => warn!("client tried to connected without handshake, it send: {other:?}"),
        Ok(None) => warn!("client closed connection immediately"),
        Err(e) => warn!("connection or decoding issue while receiving handshake: {e:?}"),
    }

    None
}

async fn handle_client(mut conn: Conn, data: Data, stats: Stats, logs: Logs) {
    loop {
        let request = match conn.try_next().await {
            Ok(Some(request)) => request,
            Ok(None) => {
                debug!("Connection ended");
                return;
            }
            Err(e) => {
                error!("Could not receive request: {e:?}");
                return;
            }
        };
        let response = match perform_request(request, &data, &stats, &logs).await {
            Err(err) => api::Response::Error(err),
            Ok(r) => r,
        };
        if let Err(e) = conn.send(response).await {
            error!("Error sending response to client: {e:?}");
            return;
        }
    }
}

async fn perform_request(
    request: api::Request,
    data: &Data,
    stats: &Stats,
    logs: &Logs,
) -> Result<api::Response, ServerError> {
    Ok(match request {
        api::Request::Handshake { .. } => return Err(ServerError::AlreadyConnected),
        api::Request::ListData => api::Response::ListData(data.list().await),
        api::Request::GetData {
            reading,
            start,
            end,
            n,
        } => {
            let (time, data) = data.get(reading, start, end, n).await?;
            api::Response::GetData { time, data }
        }
        api::Request::GetLog(device) => api::Response::GetLog(logs.get(&device).await),
        api::Request::GetStats(device) => api::Response::GetStats(stats.get(&device).await),
    })
}
