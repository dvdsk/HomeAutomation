use std::fmt;
use std::marker::Unpin;
use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::time::Duration;

use color_eyre::eyre::Context;
use color_eyre::Section;
use futures::{SinkExt, StreamExt, TryStreamExt};
use governor::clock::{Clock, DefaultClock};
use governor::{Quota, RateLimiter};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::net::{TcpListener, TcpStream};
use tokio::pin;
use tokio_serde::formats::Bincode;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use ratelimited_logger::{self as rlog, RateLimitedLogger};
use tracing::{debug, error, info, instrument};

type Conn<RpcReq, RpcResp> = tokio_serde::Framed<
    Framed<TcpStream, LengthDelimitedCodec>,
    crate::Request<RpcReq>,
    crate::Response<RpcResp>,
    Bincode<crate::Request<RpcReq>, crate::Response<RpcResp>>,
>;

pub async fn run<RpcReq, RpcResp, PerfFut>(
    port: u16,
    perform_request: impl Fn(RpcReq, &str) -> PerfFut + Clone + Send + 'static,
    sub_handler: Option<impl SubscriberHandler<Update = RpcResp> + Clone + Send + 'static>,
) -> color_eyre::Result<()>
where
    RpcReq: Unpin + Serialize + DeserializeOwned + fmt::Debug + Send + 'static,
    RpcResp: Unpin + Serialize + DeserializeOwned + fmt::Debug + Send + 'static,
    PerfFut: Future<Output = RpcResp> + Send + 'static,
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
            handshake_and_log::<RpcReq, RpcResp>(socket, source, &mut logger).await
        else {
            continue;
        };

        if let Err(allowed_again) = limiter.check_key(&(source.ip(), name.clone())) {
            let allowed_in = allowed_again.wait_time_from(DefaultClock::default().now());
            let _ignore_err = conn.send(crate::Response::TooManyReq { allowed_in }).await;
            continue;
        }

        info!("Client {name} connected from {source}");
        let Ok(()) = conn.send(crate::Response::HandshakeOk).await else {
            continue;
        };

        tokio::task::spawn(handle_client(
            conn,
            name,
            perform_request.clone(),
            sub_handler.clone(),
        ));
    }
}

async fn handshake_and_log<RpcReq, RpcResp>(
    stream: TcpStream,
    source: SocketAddr,
    logger: &mut RateLimitedLogger,
) -> Option<(Conn<RpcReq, RpcResp>, String)>
where
    RpcReq: Unpin + Serialize + DeserializeOwned + fmt::Debug + Send + 'static,
    RpcResp: Unpin + Serialize + DeserializeOwned + fmt::Debug + Send + 'static,
{
    let length_delimited = Framed::new(
        stream,
        LengthDelimitedCodec::builder()
            .max_frame_length(super::MAX_PACKAGE_SIZE)
            .new_codec(),
    );
    let mut stream: tokio_serde::Framed<_, crate::Request<RpcReq>, crate::Response<RpcResp>, _> =
        tokio_serde::Framed::new(length_delimited, Bincode::default());

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

use crate::SubscriberHandler;
#[instrument(skip(conn, perform_request, sub_handler))]
async fn handle_client<RpcReq, RpcResp, PerfFut>(
    mut conn: Conn<RpcReq, RpcResp>,
    client_name: String,
    perform_request: impl Fn(RpcReq, &str) -> PerfFut + Clone + Send + 'static,
    mut sub_handler: Option<impl SubscriberHandler<Update = RpcResp> + Send + 'static>,
) where
    RpcReq: Unpin + Serialize + DeserializeOwned + fmt::Debug + Send + 'static,
    RpcResp: Unpin + Serialize + DeserializeOwned + fmt::Debug + Send + 'static,
    PerfFut: Future<Output = RpcResp> + Send + 'static,
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
        match request {
            crate::Request::Rpc(rpc_request) => {
                let response = perform_request(rpc_request, &client_name).await;
                if let Err(e) = conn.send(crate::Response::RpcResponse(response)).await {
                    error!("Error sending response to client: {e:?}");
                    return;
                }
            }
            crate::Request::Subscribe => {
                if let Some(mut sub_handler) = sub_handler.take() {
                    let stream = sub_handler.setup().await;
                    pin!(stream);
                    if let Err(e) = conn.send(crate::Response::SubscribeOk).await {
                        error!("Error sending response to client: {e:?}");
                        return;
                    }

                    loop {
                        let Some(update) = stream.next().await else {
                            error!("Error subscribe stream should never end");
                            return;
                        };
                        if let Err(e) = conn.send(crate::Response::Update(update)).await {
                            error!("Error sending response to client: {e:?}");
                            return;
                        }
                    }
                } else {
                    error!("Got subscribe request but no subscribe possible");
                    return;
                }
            }
            crate::Request::Handshake { .. } => {
                error!("Handshake request only allowed during connect");
                return;
            }
        };
    }
}
