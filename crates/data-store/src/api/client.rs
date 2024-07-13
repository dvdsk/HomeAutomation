use futures::{SinkExt, TryStreamExt};
use tokio::net::TcpStream;
use tokio::net::ToSocketAddrs;
use tokio_serde::formats::Bincode;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use time::OffsetDateTime;

pub struct Client {
    stream: tokio_serde::Framed<
        Framed<TcpStream, LengthDelimitedCodec>,
        super::Response,
        super::Request,
        Bincode<super::Response, super::Request>,
    >,
}

#[derive(Debug, thiserror::Error)]
#[error("Error while connecting to data server: {0}")]
pub struct ConnectError(std::io::Error);

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
    pub async fn connect(addr: impl ToSocketAddrs) -> Result<Self, ConnectError> {
        let stream = TcpStream::connect(addr).await.map_err(ConnectError)?;

        let length_delimited = Framed::new(
            stream,
            LengthDelimitedCodec::builder()
                .max_frame_length(super::MAX_PACKAGE_SIZE)
                .new_codec(),
        );

        let stream = tokio_serde::Framed::new(length_delimited, Bincode::default());
        Ok(Self { stream })
    }

    async fn send_receive(&mut self, request: super::Request) -> Result<super::Response, Error> {
        self.stream.send(request).await.map_err(Error::Sending)?;
        match self.stream.try_next().await.map_err(Error::Receiving)? {
            Some(super::Response::Error(e)) => Err(Error::Server(e)),
            Some(response) => Ok(response),
            None => Err(Error::ConnectionClosed),
        }
    }

    pub async fn list_data(&mut self) -> Result<Vec<protocol::Reading>, Error> {
        let request = super::Request::ListData;
        match self.send_receive(request.clone()).await? {
            super::Response::ListData(list) => Ok(list),
            response => Err(Error::IncorrectResponse {
                request: format!("{request:?}"),
                response: format!("{response:?}"),
            }),
        }
    }

    pub async fn get_data(
        &mut self,
        start: OffsetDateTime,
        end: OffsetDateTime,
        reading: protocol::Reading,
        n: usize,
    ) -> Result<(Vec<OffsetDateTime>, Vec<f32>), Error> {
        let request = super::Request::GetData {
            reading,
            start,
            end,
            n,
        };
        match self.send_receive(request.clone()).await? {
            super::Response::GetData { time, data } => Ok((time, data)),
            response => Err(Error::IncorrectResponse {
                request: format!("{request:?}"),
                response: format!("{response:?}"),
            }),
        }
    }
}
