use std::io::{BufRead, BufReader};
use std::net::{TcpStream, ToSocketAddrs};

use protocol::{DecodeError, SensorMessage};
use tokio::io::AsyncBufReadExt;
use tracing::trace;

pub struct Subscriber {
    reader: BufReader<TcpStream>,
    buf: Vec<u8>,
    values: std::vec::IntoIter<Msg>,
}

type Msg = Result<protocol::Reading, protocol::Error>;

impl Subscriber {
    pub fn connect(addr: impl ToSocketAddrs) -> Result<Self, SubscribeError> {
        let conn = TcpStream::connect(addr).map_err(SubscribeError::ConnFailed)?;
        let reader = BufReader::new(conn);
        let buf = Vec::new();

        Ok(Self {
            reader,
            buf,
            values: vec![].into_iter(),
        })
    }

    pub fn next(&mut self) -> Result<Msg, SubscribeError> {
        if let Some(val) = self.values.next() {
            return Ok(val);
        }

        let n_read = self
            .reader
            .read_until(0, &mut self.buf)
            .map_err(SubscribeError::ConnFailed)?;

        if n_read == 0 {
            return Err(SubscribeError::ConnEnded)?;
        }

        self.buf.resize(n_read, 0); // ensure stop delimiter in bytes
        let msg = &mut self.buf[0..n_read];
        trace!("{:?}", &msg);
        let msg = SensorMessage::<50>::decode(msg).map_err(SubscribeError::DecodeFailed)?;
        self.buf.clear();

        self.values = msg.values.to_vec().into_iter();
        Ok(self
            .values
            .next()
            .expect("min values in sensormessage is one"))
    }
}

pub struct AsyncSubscriber {
    reader: tokio::io::BufReader<tokio::net::TcpStream>,
    buf: Vec<u8>,
    values: std::vec::IntoIter<Msg>,
}

impl AsyncSubscriber {
    pub async fn connect(addr: impl tokio::net::ToSocketAddrs) -> Result<Self, SubscribeError> {
        let conn = tokio::net::TcpStream::connect(addr)
            .await
            .map_err(SubscribeError::ConnFailed)?;
        let reader = tokio::io::BufReader::new(conn);
        let buf = Vec::new();

        Ok(Self {
            reader,
            buf,
            values: vec![].into_iter(),
        })
    }

    pub async fn next(&mut self) -> Result<Msg, SubscribeError> {
        if let Some(val) = self.values.next() {
            return Ok(val);
        }

        let n_read = self
            .reader
            .read_until(0, &mut self.buf)
            .await
            .map_err(SubscribeError::ConnFailed)?;

        if n_read == 0 {
            return Err(SubscribeError::ConnEnded)?;
        }

        self.buf.resize(n_read, 0); // ensure stop delimiter in bytes
        let msg = &mut self.buf[0..n_read];
        trace!("{:?}", &msg);
        let msg = SensorMessage::<50>::decode(msg).map_err(SubscribeError::DecodeFailed)?;
        self.buf.clear();

        self.values = msg.values.to_vec().into_iter();
        Ok(self
            .values
            .next()
            .expect("min values in sensormessage is one"))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SubscribeError {
    #[error("The connection to the subscribe server failed, error: {0}")]
    ConnFailed(std::io::Error),
    #[error("Could not decode message, is protocol lib up to date on server and client? Decoderror: {0:?}")]
    DecodeFailed(DecodeError),
    #[error("Connection ended")]
    ConnEnded,
}
