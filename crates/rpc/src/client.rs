use core::fmt;
use std::marker::Unpin;
use std::time::Duration;

use futures::{SinkExt, TryStreamExt};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio::net::ToSocketAddrs;
use tokio::time::error::Elapsed;
use tokio::time::timeout_at;
use tokio::time::Instant;
use tokio_serde::formats::Bincode;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::Request;
use crate::Response;

pub struct RpcClient<RpcReq, RpcResp>
where
    RpcReq: Serialize,
    RpcResp: Serialize,
{
    stream: tokio_serde::Framed<
        Framed<TcpStream, LengthDelimitedCodec>,
        Response<RpcResp>,
        Request<RpcReq>,
        Bincode<Response<RpcResp>, Request<RpcReq>>,
    >,
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("Error while connecting to server: {0}")]
    Io(std::io::Error),
    #[error("Could not send handshake: {0}")]
    Sending(std::io::Error),
    #[error("Timed out waiting for server")]
    Timeout,
    #[error("Server unexpectedly closed the connection")]
    Closed,
    #[error("Could not receive server response: {0:?}")]
    Receiving(std::io::Error),
    #[error("Client was already connected")]
    AlreadyConnected,
}

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error("Got unexpected response response to request {request:?}")]
    IncorrectResponse { request: String, response: String },
    #[error("Error while sending request: {0}")]
    Sending(std::io::Error),
    #[error("Error while sending request: {0}")]
    Receiving(std::io::Error),
    #[error("Server closed connection before it awnserd")]
    ConnectionClosed,
}

impl<RpcReq, RpcResp> RpcClient<RpcReq, RpcResp>
where
    RpcReq: Unpin + Serialize + fmt::Debug,
    RpcResp: Unpin + Serialize + DeserializeOwned + fmt::Debug,
{
    pub async fn connect(addr: impl ToSocketAddrs, name: String) -> Result<Self, ConnectError> {
        let stream = TcpStream::connect(addr).await.map_err(ConnectError::Io)?;

        let length_delimited = Framed::new(
            stream,
            LengthDelimitedCodec::builder()
                .max_frame_length(super::MAX_PACKAGE_SIZE)
                .new_codec(),
        );

        let mut stream = tokio_serde::Framed::new(length_delimited, Bincode::default());
        stream
            .send(Request::Handshake { client_name: name })
            .await
            .map_err(ConnectError::Sending)?;

        match tokio::time::timeout(Duration::from_secs(2), stream.try_next()).await {
            Ok(Ok(Some(Response::HandshakeOk))) => Ok(Self { stream }),
            Ok(Ok(Some(Response::AlreadyConnected))) => Err(ConnectError::AlreadyConnected),
            Ok(Ok(Some(other))) => unreachable!(
                "Server should return handshake response or error after sending \
                first handshake, got impossible response: {other:?}"
            ),
            Ok(Ok(None)) => Err(ConnectError::Closed),
            Ok(Err(e)) => Err(ConnectError::Receiving(e)),
            Err(_) => Err(ConnectError::Timeout),
        }
    }

    pub async fn send_receive(&mut self, request: RpcReq) -> Result<RpcResp, RpcError> {
        fn send_timeout_err(_: Elapsed) -> RpcError {
            RpcError::Sending(std::io::Error::new(std::io::ErrorKind::TimedOut, ""))
        }
        fn receive_timeout_err(_: Elapsed) -> RpcError {
            RpcError::Receiving(std::io::Error::new(std::io::ErrorKind::TimedOut, ""))
        }

        let deadline = Instant::now() + Duration::from_secs(5);
        let request = Request::Rpc(request);
        timeout_at(deadline, self.stream.send(request))
            .await
            .map_err(send_timeout_err)?
            .map_err(RpcError::Sending)?;
        match timeout_at(deadline, self.stream.try_next())
            .await
            .map_err(receive_timeout_err)?
            .map_err(RpcError::Receiving)?
        {
            Some(Response::AlreadyConnected) => {
                unreachable!("not creating a connection")
            }
            Some(Response::RpcResponse(v)) => Ok(v),
            Some(Response::TooManyReq { .. }) => {
                unreachable!("only creating connections is ratelimited")
            }
            Some(Response::HandshakeOk) => unreachable!("is only send during initial connection"),
            None => Err(RpcError::ConnectionClosed),
        }
    }
}
