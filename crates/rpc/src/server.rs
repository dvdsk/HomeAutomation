use std::fmt;
use std::marker::Unpin;
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::time::Duration;

use color_eyre::eyre::Context;
use color_eyre::Section;
use futures::{SinkExt, TryStreamExt};
use governor::clock::{Clock, DefaultClock};
use governor::{Quota, RateLimiter};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::net::{TcpListener, TcpStream};
use tokio_serde::formats::Bincode;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use ratelimited_logger::{self as rlog, RateLimitedLogger};
use tracing::{debug, error, info};

type Conn<RpcReq, RpcResp, RpcErr> = tokio_serde::Framed<
    Framed<TcpStream, LengthDelimitedCodec>,
    crate::Request<RpcReq>,
    crate::Response<RpcResp, RpcErr>,
    Bincode<crate::Request<RpcReq>, crate::Response<RpcResp, RpcErr>>,
>;

pub async fn run<RpcReq, RpcResp, RpcErr, Fut>(
    port: u16,
    perform_request: impl FnOnce(RpcReq) -> Fut + Clone + Send + 'static,
) -> color_eyre::Result<()>
where
    RpcReq: Unpin + Serialize + DeserializeOwned + fmt::Debug + Send + 'static,
    RpcResp: Unpin + Serialize + DeserializeOwned + fmt::Debug + Send + 'static,
    RpcErr: Unpin + Serialize + DeserializeOwned + fmt::Debug + Send + std::error::Error + 'static,
    Fut: Future<Output = RpcResp> + Send + 'static,
{
    let quota = Quota::with_period(Duration::from_secs(1))
        .unwrap()
        .allow_burst(NonZeroU32::new(5u32).unwrap());
    let limiter = RateLimiter::keyed(quota);
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .wrap_err("Could not bind to address")
        .with_note(|| format!("port: {port}"))?;
    let mut logger = RateLimitedLogger::new();

    loop {
        let (socket, source) = match listener.accept().await {
            Err(e) => {
                rlog::warn!(logger; "client could not connect: {e}");
                continue;
            }
            Ok(res) => res,
        };

        let Some((mut conn, name)) =
            handshake_and_log::<RpcReq, RpcResp, RpcErr>(socket, source, &mut logger).await
        else {
            continue;
        };

        if let Err(allowed_again) = limiter.check_key(&(source.ip(), name.clone())) {
            let allowed_in = allowed_again.wait_time_from(DefaultClock::default().now());
            let _ignore_err = conn.send(crate::Response::TooManyReq { allowed_in });
            continue;
        }

        info!("Client {name} connected from {source}");
        let Ok(()) = conn.send(crate::Response::HandshakeOk).await else {
            continue;
        };

        tokio::task::spawn(handle_client(conn, perform_request.clone()));
    }
}

async fn handshake_and_log<RpcReq, RpcResp, RpcErr>(
    stream: TcpStream,
    source: SocketAddr,
    logger: &mut RateLimitedLogger,
) -> Option<(Conn<RpcReq, RpcResp, RpcErr>, String)>
where
    RpcReq: Unpin + Serialize + DeserializeOwned + fmt::Debug + Send + 'static,
    RpcResp: Unpin + Serialize + DeserializeOwned + fmt::Debug + Send + 'static,
    RpcErr: Unpin + Serialize + DeserializeOwned + fmt::Debug + Send + std::error::Error + 'static,
{
    let length_delimited = Framed::new(
        stream,
        LengthDelimitedCodec::builder()
            .max_frame_length(1024)
            .new_codec(),
    );
    let mut stream: tokio_serde::Framed<
        _,
        crate::Request<RpcReq>,
        crate::Response<RpcResp, RpcErr>,
        _,
    > = tokio_serde::Framed::new(length_delimited, Bincode::default());

    match stream.try_next().await {
        Ok(Some(crate::Request::Handshake { client_name })) => {
            return Some((stream, client_name));
        }
        Ok(Some(other)) => {
            rlog::warn!(logger; "client from {source} tried to connected without handshake, it send: {other:?}")
        }
        Ok(None) => rlog::warn!(logger; "client from {source} closed connection immediately"),
        Err(e) => {
            rlog::warn!(logger; "connection or decoding issue while receiving handshake from {source}, error: {e:?}")
        }
    }

    None
}

use core::future::Future;
async fn handle_client<RpcReq, RpcResp, RpcErr, Fut>(
    mut conn: Conn<RpcReq, RpcResp, RpcErr>,
    perform_request: impl FnOnce(RpcReq) -> Fut + Clone + Send + 'static,
) where
    RpcReq: Unpin + Serialize + DeserializeOwned + fmt::Debug + Send + 'static,
    RpcResp: Unpin + Serialize + DeserializeOwned + fmt::Debug + Send + 'static,
    RpcErr: Unpin + Serialize + DeserializeOwned + fmt::Debug + Send + std::error::Error + 'static,
    Fut: Future<Output = RpcResp> + Send + 'static,
{
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
        let crate::Request::Rpc(rpc_request) = request else {
            error!("Handshake request is not allowed after we send back HandshakeOk");
            return;
        };

        let response = perform_request.clone()(rpc_request).await;
        if let Err(e) = conn.send(crate::Response::RpcResponse(response)).await {
            error!("Error sending response to client: {e:?}");
            return;
        }
    }
}
