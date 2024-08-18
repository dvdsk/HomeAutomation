use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;
use std::vec;

use protocol::Msg;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tracing::instrument;

pub mod reconnecting;


pub struct Subscriber {
    reader: BufReader<TcpStream>,
    buf: Vec<u8>,
    values: vec::IntoIter<SubMessage>,
}

impl Subscriber {
    pub fn connect(addr: impl Into<SocketAddr>, name: &str) -> Result<Self, SubscribeError> {
        let mut conn = TcpStream::connect_timeout(&addr.into(), Duration::from_millis(400))
            .map_err(SubscribeError::ConnFailed)?;
        let name_len: u8 = name
            .len()
            .try_into()
            .map_err(|_| SubscribeError::NameTooLong)?;
        conn.write_all(&[name_len])
            .map_err(SubscribeError::FailedToWriteName)?;
        conn.write_all(name.as_bytes())
            .map_err(SubscribeError::FailedToWriteName)?;
        let reader = BufReader::new(conn);
        let buf = Vec::new();

        Ok(Self {
            reader,
            buf,
            values: vec![].into_iter(),
        })
    }

    pub fn next_msg(&mut self) -> Result<SubMessage, SubscribeError> {
        if let Some(val) = self.values.next() {
            return Ok(val);
        }

        self.buf.clear();
        let n_read = self
            .reader
            .read_until(0, &mut self.buf)
            .map_err(SubscribeError::ConnFailed)?;
        decode_buffer_and_return_first(n_read, &mut self.buf, &mut self.values)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct AsyncSubscriber {
    reader: tokio::io::BufReader<tokio::net::TcpStream>,
    buf: Vec<u8>,
    values: vec::IntoIter<SubMessage>,
}

impl AsyncSubscriber {
    pub async fn connect(
        addr: impl tokio::net::ToSocketAddrs,
        name: &str,
    ) -> Result<Self, SubscribeError> {
        let mut conn = tokio::net::TcpStream::connect(addr)
            .await
            .map_err(SubscribeError::ConnFailed)?;
        let name_len: u8 = name
            .len()
            .try_into()
            .map_err(|_| SubscribeError::NameTooLong)?;
        conn.write_all(&[name_len])
            .await
            .map_err(SubscribeError::FailedToWriteName)?;
        conn.write_all(name.as_bytes())
            .await
            .map_err(SubscribeError::FailedToWriteName)?;
        let reader = tokio::io::BufReader::new(conn);
        let buf = Vec::new();

        Ok(Self {
            reader,
            buf,
            values: vec![].into_iter(),
        })
    }

    pub async fn next_msg(&mut self) -> Result<SubMessage, SubscribeError> {
        if let Some(val) = self.values.next() {
            return Ok(val);
        }

        self.buf.clear();
        let n_read = self
            .reader
            .read_until(0, &mut self.buf)
            .await
            .map_err(SubscribeError::ConnFailed)?;
        decode_buffer_and_return_first(n_read, &mut self.buf, &mut self.values)
    }
}

#[instrument(level = "trace")]
fn decode_buffer_and_return_first(
    n_read: usize,
    buf: &mut Vec<u8>,
    buffer: &mut vec::IntoIter<SubMessage>,
) -> Result<SubMessage, SubscribeError> {
    assert!(buffer.next().is_none());

    if n_read == 0 {
        return Err(SubscribeError::ConnEnded);
    }

    buf.resize(n_read, 0); // ensure stop delimiter in bytes
    let bytes = &mut buf[0..n_read];
    let decoded: Msg<50> = Msg::decode(bytes).map_err(SubscribeError::DecodeFailed)?;
    match decoded {
        Msg::Readings(readings) => {
            *buffer = readings
                .values
                .iter()
                .map(Clone::clone)
                .map(SubMessage::Reading)
                .collect::<Vec<_>>()
                .into_iter();
        }
        Msg::ErrorReport(report) => {
            *buffer = vec![SubMessage::ErrorReport(Box::new(report.error))].into_iter();
        }
    }

    Ok(buffer.next().expect("min values in sensormessage is one"))
}

#[derive(Debug, thiserror::Error)]
pub enum SubscribeError {
    #[error("The connection to the subscribe server failed, error: {0}")]
    ConnFailed(std::io::Error),
    #[error(
        "Could not decode message, is protocol lib up to date on server \
        and client? Or has the server crashed? Decoderror: {0:?}"
    )]
    DecodeFailed(protocol::DecodeMsgError),
    #[error("Connection ended")]
    ConnEnded,
    #[error("A subscribers name must be smaller then 256 bytes long")]
    NameTooLong,
    #[error("Could not send name to data-server: {0}")]
    FailedToWriteName(std::io::Error),
}
