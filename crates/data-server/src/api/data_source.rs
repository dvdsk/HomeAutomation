use protocol::{affector, DecodeMsgError};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream, ToSocketAddrs};

pub mod reconnecting;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not connect to data-server: {0}")]
    Connecting(std::io::Error),
    #[error("Could not send handshake to data-server: {0}")]
    Handshake(std::io::Error),
    #[error("Too many affectors, max: {max}, requires: {requires}")]
    TooManyAffectors { max: usize, requires: usize },
}

pub struct Client {
    sender: Sender,
    receiver: Receiver,
}

impl Client {
    /// Needs a list of the affectors that can be controlled through this
    /// node as an argument. If your node provides not controllable affectors
    /// pass in an empty Vec.
    pub async fn connect(
        addr: impl ToSocketAddrs,
        affectors: Vec<protocol::Affector>,
    ) -> Result<Self, Error> {
        let mut stream = TcpStream::connect(addr).await.map_err(Error::Connecting)?;
        let mut list = protocol::affector::ListMessage::<50>::empty();

        if affectors.len() > list.values.capacity() {
            return Err(Error::TooManyAffectors {
                max: list.values.capacity(),
                requires: affectors.len(),
            });
        }
        for affector in affectors {
            list.values.push(affector).expect("Is checked right above");
        }
        let handshake = protocol::Msg::AffectorList(list);
        let handshake = handshake.encode();
        stream
            .write_all(&handshake)
            .await
            .map_err(Error::Handshake)?;

        let (reader, writer) = stream.into_split();

        Ok(Self {
            receiver: Receiver {
                reader,
                decoder: affector::Decoder::default(),
                buffer: vec![0u8; 100],
            },
            sender: Sender(writer),
        })
    }

    /// # Cancel safety
    /// This is cancel safe, if it is canceled the reading has not been send yet
    pub async fn send_reading(&mut self, reading: protocol::Reading) -> Result<(), std::io::Error> {
        self.sender.send_reading(reading).await
    }

    /// # Cancel safety
    /// This is cancel safe, if it is canceled the report has not been send yet
    pub async fn send_error(&mut self, report: protocol::Error) -> Result<(), std::io::Error> {
        self.sender.send_error(report).await
    }

    /// Checks if the encoded message can be decoded then sends it.
    pub async fn check_send_encoded(&mut self, msg: &[u8]) -> Result<(), SendPreEncodedError> {
        protocol::Msg::<50>::decode(msg.to_vec()).map_err(SendPreEncodedError::EncodingCheck)?;
        self.send_bytes(msg).await.map_err(SendPreEncodedError::Io)
    }

    pub(crate) async fn send_bytes(&mut self, bytes: &[u8]) -> Result<(), std::io::Error> {
        self.sender.send_bytes(bytes).await
    }

    pub fn split(self) -> (Sender, Receiver) {
        (self.sender, self.receiver)
    }
}

pub struct Sender(OwnedWriteHalf);
pub struct Receiver {
    reader: OwnedReadHalf,
    decoder: affector::Decoder,
    buffer: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum SendPreEncodedError {
    #[error("Ran into io error while sending pre-encoded msg: {0}")]
    Io(std::io::Error),
    #[error("Pre-encoded message, could not be decoded it might be from a previous version: {0}")]
    EncodingCheck(DecodeMsgError),
}

impl Sender {
    pub(crate) async fn send_bytes(&mut self, bytes: &[u8]) -> Result<(), std::io::Error> {
        self.0.write_all(bytes).await
    }

    pub async fn send_reading(&mut self, reading: protocol::Reading) -> Result<(), std::io::Error> {
        let mut readings = protocol::SensorMessage::<1>::default();
        readings
            .values
            .push(reading)
            .expect("capacity allows one push");
        let msg = protocol::Msg::Readings(readings);
        let bytes = msg.encode();

        self.0.write_all(&bytes).await
    }

    pub async fn send_error(&mut self, report: protocol::Error) -> Result<(), std::io::Error> {
        let msg = protocol::Msg::<1>::ErrorReport(protocol::ErrorReport::new(report));
        let bytes = msg.encode();

        self.0.write_all(&bytes).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReceiveError {
    #[error("General io error while receiving data from data-server")]
    Io(std::io::Error),
    #[error("The connection was closed by the data-server")]
    ConnClosed,
}

impl Receiver {
    pub async fn receive(&mut self) -> Result<protocol::Affector, ReceiveError> {
        loop {
            if !self.buffer.is_empty() {
                if let Some((item, remaining)) = self.decoder.feed(&self.buffer) {
                    let new_buffer = remaining.to_vec();
                    self.buffer = new_buffer;
                    return Ok(item);
                }
            }

            self.buffer.resize(100, 0);
            let res = self.reader.read(&mut self.buffer).await;

            let n_read = match res {
                Ok(0) => return Err(ReceiveError::ConnClosed),
                Ok(n_read) => n_read,
                Err(e) => return Err(ReceiveError::Io(e)),
            };
            self.buffer.truncate(n_read);
        }
    }
}
