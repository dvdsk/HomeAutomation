use color_eyre::eyre::Context;
use color_eyre::Section;
use futures::{SinkExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_serde::formats::Bincode;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{debug, error, info, warn};

use super::db::Data;
use crate::api::{self, ServerError};

pub(crate) async fn handle(port: u16, data: Data) -> color_eyre::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .wrap_err("Could not bind to address")
        .with_note(|| format!("port: {port}"))?;

    loop {
        let socket = match listener.accept().await {
            Ok((socket, source)) => {
                info!("client connected from source: {source}");
                socket
            }
            Err(e) => {
                warn!("client could not connect: {e}");
                continue;
            }
        };

        tokio::task::spawn(handle_client(socket, data.clone()));
    }
}

pub(crate) async fn handle_client(socket: TcpStream, data: Data) {
    let length_delimited = Framed::new(
        socket,
        LengthDelimitedCodec::builder()
            .max_frame_length(api::MAX_PACKAGE_SIZE)
            .new_codec(),
    );
    let mut stream: tokio_serde::Framed<_, api::Request, api::Response, _> =
        tokio_serde::Framed::new(length_delimited, Bincode::default());

    loop {
        let request = match stream.try_next().await {
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
        let response = match perform_request(request, &data).await {
            Err(err) => api::Response::Error(err),
            Ok(r) => r,
        };
        if let Err(e) = stream.send(response).await {
            error!("Error sending response to client: {e:?}");
            return;
        }
    }
}

async fn perform_request(request: api::Request, data: &Data) -> Result<api::Response, ServerError> {
    Ok(match request {
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
    })
}
