use std::time::Duration;

use futures::{SinkExt, TryStreamExt};
use tokio::net::TcpStream;
use tokio::net::ToSocketAddrs;
use tokio_serde::formats::Bincode;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use super::Request;
use super::Response;

pub struct Client {
    stream: tokio_serde::Framed<
        Framed<TcpStream, LengthDelimitedCodec>,
        Response,
        super::Request,
        Bincode<Response, super::Request>,
    >,
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("Error while connecting to data server: {0}")]
    Io(std::io::Error),
    #[error("Could not send handshake: {0}")]
    Sending(std::io::Error),
    #[error("Timed out waiting for ")]
    Timeout,
    #[error("Server unexpectedly closed the connection")]
    Closed,
    #[error("Server send back an error: {0:?}")]
    ServerError(super::ServerError),
    #[error("Could not receive server response: {0:?}")]
    Receiving(std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Got unexpected response response to request {request:?}")]
    IncorrectResponse { request: String, response: String },
    #[error("Error while sending request: {0}")]
    Sending(std::io::Error),
    #[error("Error while sending request: {0}")]
    Receiving(std::io::Error),
    #[error("Server ran into an error while processing our request: {0}")]
    Server(super::ServerError),
    #[error("Server closed connection before it awnserd")]
    ConnectionClosed,
}

impl Client {
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
            .send(Request::Handshake { name })
            .await
            .map_err(ConnectError::Sending)?;

        match tokio::time::timeout(Duration::from_secs(2), stream.try_next()).await {
            Ok(Ok(Some(Response::Handshake))) => Ok(Self { stream }),
            Ok(Ok(Some(Response::Error(e)))) => Err(ConnectError::ServerError(e)),
            Ok(Ok(Some(other))) => unreachable!(
                "Server should return handshake response or error after sending \
                first handshake, got impossible response: {other:?}"
            ),
            Ok(Ok(None)) => Err(ConnectError::Closed),
            Ok(Err(e)) => Err(ConnectError::Receiving(e)),
            Err(_) => Err(ConnectError::Timeout),
        }
    }

    async fn send_receive(&mut self, request: super::Request) -> Result<Response, Error> {
        self.stream.send(request).await.map_err(Error::Sending)?;
        match self.stream.try_next().await.map_err(Error::Receiving)? {
            Some(Response::Error(e)) => Err(Error::Server(e)),
            Some(response) => Ok(response),
            None => Err(Error::ConnectionClosed),
        }
    }
}
