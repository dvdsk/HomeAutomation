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

type Stream<RpcReq, RpcResp> = tokio_serde::Framed<
    Framed<TcpStream, LengthDelimitedCodec>,
    Response<RpcResp>,
    Request<RpcReq>,
    Bincode<Response<RpcResp>, Request<RpcReq>>,
>;

pub struct RpcClient<RpcReq, RpcResp>
where
    RpcResp: Serialize,
{
    stream: Stream<RpcReq, RpcResp>,
}

impl<T, V: Serialize> fmt::Debug for RpcClient<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RpcClient").finish()
    }
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
    #[error("Client tried to connected too many times, allowed again in: {0:?}")]
    RateLimited(Duration),
}

#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error("Got unexpected response response to request.\nRequest was: {request:?},\nGot response: {response:?}")]
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
    async fn try_connect(
        addr: impl ToSocketAddrs,
        name: String,
    ) -> Result<Stream<RpcReq, RpcResp>, ConnectError> {
        let stream = TcpStream::connect(addr).await.map_err(ConnectError::Io)?;
        let _ignore_error = stream.set_nodelay(true);

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
        Ok(stream)
    }

    pub async fn connect(addr: impl ToSocketAddrs, name: String) -> Result<Self, ConnectError> {
        let mut stream = Self::try_connect(addr, name).await?;
        match tokio::time::timeout(Duration::from_secs(2), stream.try_next()).await {
            Ok(Ok(Some(Response::HandshakeOk))) => Ok(Self { stream }),
            Ok(Ok(Some(Response::AlreadyConnected))) => Err(ConnectError::AlreadyConnected),
            Ok(Ok(Some(Response::TooManyReq { allowed_in }))) => {
                Err(ConnectError::RateLimited(allowed_in))
            }

            Ok(Ok(Some(other))) => unreachable!(
                "Server should return handshake response or error after sending \
                first handshake, got impossible response: {other:?}"
            ),
            Ok(Ok(None)) => Err(ConnectError::Closed),
            Ok(Err(e)) => Err(ConnectError::Receiving(e)),
            Err(_) => Err(ConnectError::Timeout),
        }
    }

    pub async fn subscribe(&mut self) -> Result<(), RpcError> {
        fn send_timeout_err(_: Elapsed) -> RpcError {
            RpcError::Sending(std::io::Error::new(std::io::ErrorKind::TimedOut, ""))
        }
        fn receive_timeout_err(_: Elapsed) -> RpcError {
            RpcError::Receiving(std::io::Error::new(std::io::ErrorKind::TimedOut, ""))
        }

        let deadline = Instant::now() + Duration::from_secs(5);
        let request = Request::Subscribe;
        timeout_at(deadline, self.stream.send(request))
            .await
            .map_err(send_timeout_err)?
            .map_err(RpcError::Sending)?;

        let response = timeout_at(deadline, self.stream.try_next())
            .await
            .map_err(receive_timeout_err)?
            .map_err(RpcError::Receiving)?
            .ok_or(RpcError::ConnectionClosed)?;
        match response {
            Response::AlreadyConnected
            | Response::RpcResponse(_)
            | Response::Update(_)
            | Response::TooManyReq { .. }
            | Response::HandshakeOk => {
                unreachable!("incorrect response to subscribe: {response:?}")
            }
            Response::SubscribeOk => Ok(()),
        }
    }

    pub async fn next(&mut self) -> Result<RpcResp, RpcError> {
        let received = self
            .stream
            .try_next()
            .await
            .map_err(RpcError::Receiving)?
            .ok_or(RpcError::ConnectionClosed)?;

        match received {
            Response::AlreadyConnected
            | Response::RpcResponse(_)
            | Response::TooManyReq { .. }
            | Response::HandshakeOk
            | Response::SubscribeOk => {
                unreachable!("incorrect response to get next, are we subscribed?")
            }
            Response::Update(v) => Ok(v),
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
            .ok_or(RpcError::ConnectionClosed)?
        {
            Response::AlreadyConnected
            | Response::Update(_)
            | Response::TooManyReq { .. }
            | Response::HandshakeOk
            | Response::SubscribeOk => unreachable!("only expected after a subscribe"),
            Response::RpcResponse(v) => Ok(v),
        }
    }
}
