use std::net::SocketAddr;
use std::time::Duration;

use futures::SinkExt;
use tokio::net::TcpStream;
use tokio::time::error::Elapsed;
use tokio::time::timeout_at;
use tokio::time::Instant;
use tokio_serde::formats::Bincode;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use super::AffectorError;
use super::Request;
use super::Response;
use super::ServerError;
use super::SubMessage;
use super::SubscribeError;

pub(crate) mod reconnecting;

#[derive(Debug)]
pub struct Client {
    stream: tokio_serde::Framed<
        Framed<TcpStream, LengthDelimitedCodec>,
        Response,
        Request,
        Bincode<Response, Request>,
    >,
}

impl Client {
    pub async fn connect(addr: impl Into<SocketAddr>, name: String) -> Result<Self, ConnectError> {
        let stream = TcpStream::connect(addr.into())
            .await
            .map_err(ConnectError::Io)?;

        let length_delimited = Framed::new(
            stream,
            LengthDelimitedCodec::builder()
                .max_frame_length(1024)
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

    async fn send_receive<T: std::error::Error>(
        &mut self,
        request: Request,
    ) -> Result<Response, Error<T>> {
        fn send_timeout_err<T: std::error::Error>(_: Elapsed) -> Error<T> {
            Error::Sending(std::io::Error::new(std::io::ErrorKind::TimedOut, ""))
        }
        fn receive_timeout_err<T: std::error::Error>(_: Elapsed) -> Error<T> {
            Error::Receiving(std::io::Error::new(std::io::ErrorKind::TimedOut, ""))
        }

        let deadline = Instant::now() + Duration::from_secs(5);
        timeout_at(deadline, self.stream.send(request))
            .await
            .map_err(send_timeout_err)?
            .map_err(Error::Sending)?;
        match timeout_at(deadline, self.stream.try_next())
            .await
            .map_err(receive_timeout_err)?
            .map_err(Error::Receiving)?
        {
            Some(Response::Error(e)) => Err(Error::Server(e)),
            Some(response) => Ok(response),
            None => Err(Error::ConnectionClosed),
        }
    }

    pub async fn actuate_affector(
        &mut self,
        affector: protocol::Affector,
    ) -> Result<(), Error<AffectorError>> {
        let request = Request::Actuate(affector);
        match self.send_receive(request.clone()).await {
            Ok(Response::Actuate(res)) => res.map_err(Error::Request),
            Err(err) => Err(err),
            response => Err(Error::IncorrectResponse {
                request: format!("{request:?}"),
                response: format!("{response:?}"),
            }),
        }
    }

    pub async fn list_affectors(
        &mut self,
    ) -> Result<Vec<protocol::Affector>, Error<AffectorError>> {
        let request = Request::ListAffectors;
        match self.send_receive(request.clone()).await {
            Ok(Response::ListAffectors(list)) => Ok(list),
            Err(err) => Err(err),
            response => Err(Error::IncorrectResponse {
                request: format!("{request:?}"),
                response: format!("{response:?}"),
            }),
        }
    }

    pub async fn subscribe(mut self) -> Result<Subscribed, Error<SubscribeError>> {
        let request = Request::Subscribe;
        match self.send_receive(request.clone()).await {
            Ok(Response::Subscribe) => Ok(Subscribed(self)),
            Err(err) => Err(err),
            response => Err(Error::IncorrectResponse {
                request: format!("{request:?}"),
                response: format!("{response:?}"),
            }),
        }
    }
}

#[derive(Debug)]
pub struct Subscribed(Client);

impl Subscribed {
    pub async fn next(&mut self) -> Result<SubMessage, Error<SubscribeError>> {
        let received = self
            .0
            .stream
            .try_next()
            .await
            .map_err(Error::Receiving)?
            .ok_or(Error::ConnectionClosed)?;

        if let Response::SubUpdate(update) = received {
            Ok(update)
        } else {
            Err(Error::IncorrectResponse {
                request: "none, we are subscribed".to_string(),
                response: format!("{received:?}"),
            })
        }
    }
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
    ServerError(ServerError),
    #[error("Could not receive server response: {0:?}")]
    Receiving(std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum Error<T: std::error::Error> {
    #[error("Got unexpected response response to request {request:?}")]
    IncorrectResponse { request: String, response: String },
    #[error("Error while sending request: {0}")]
    Sending(std::io::Error),
    #[error("Error while sending request: {0}")]
    Receiving(std::io::Error),
    #[error("General error while processing request")]
    Server(ServerError),
    #[error("Server ran into an specific error with our request: {0}")]
    Request(#[from] T),
    #[error("Server closed connection before it awnserd")]
    ConnectionClosed,
}
