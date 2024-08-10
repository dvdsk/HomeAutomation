use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::time::Duration;

use color_eyre::eyre::Context;
use color_eyre::Section;
use futures::{SinkExt, TryStreamExt};
use governor::clock::{Clock, DefaultClock};
use governor::{Quota, RateLimiter};
use tokio::net::{TcpListener, TcpStream};
use tokio_serde::formats::Bincode;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{debug, error, info, warn};

use super::db::Data;
use crate::api::{self, ServerError};

type Conn = tokio_serde::Framed<
    Framed<TcpStream, LengthDelimitedCodec>,
    api::Request,
    api::Response,
    Bincode<api::Request, api::Response>,
>;

pub(crate) async fn handle(port: u16, data: Data) -> color_eyre::Result<()> {
    let quota = Quota::with_period(Duration::from_secs(1))
        .unwrap()
        .allow_burst(NonZeroU32::new(5u32).unwrap());
    let limiter = RateLimiter::keyed(quota);
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

        let Some((mut conn, name)) = handshake_and_log(socket, source).await else {
            continue;
        };

        if let Err(allowed_again) = limiter.check_key(&(source, name.clone())) {
            let allowed_in = allowed_again.wait_time_from(DefaultClock::default().now());
            let _ignore_err = conn.send(api::Response::Error(ServerError::TooManyRequests(
                allowed_in,
            )));
            continue;
        }

        info!("Client {name} connected from {source}");
        let Ok(()) = conn.send(api::Response::Handshake).await else {
            continue;
        };

        tokio::task::spawn(handle_client(conn, data.clone()));
    }
}

async fn handshake_and_log(stream: TcpStream, source: SocketAddr) -> Option<(Conn, String)> {
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
            return Some((stream, name));
        }
        Ok(Some(other)) => {
            warn!("client from {source} tried to connected without handshake, it send: {other:?}")
        }
        Ok(None) => warn!("client from {source} closed connection immediately"),
        Err(e) => warn!("connection or decoding issue while receiving handshake from {source}, error: {e:?}"),
    }

    None
}

async fn handle_client(mut conn: Conn, data: Data) {
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
        let response = match perform_request(request, &data).await {
            Err(err) => api::Response::Error(err),
            Ok(r) => r,
        };
        if let Err(e) = conn.send(response).await {
            error!("Error sending response to client: {e:?}");
            return;
        }
    }
}

async fn perform_request(request: api::Request, data: &Data) -> Result<api::Response, ServerError> {
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
    })
}
