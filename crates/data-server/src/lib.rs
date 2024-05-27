use std::io::{BufRead, BufReader};
use std::net::{TcpStream, ToSocketAddrs};

use protocol::{DecodeError, SensorMessage};

pub struct Subscriber {
    reader: BufReader<TcpStream>,
    buf: Vec<u8>,
    values: std::vec::IntoIter<Msg>,
}

type Msg = Result<protocol::Reading, protocol::Error>;

impl Subscriber {
    pub fn connect(addr: impl ToSocketAddrs) -> Result<Self, std::io::Error> {
        let conn = TcpStream::connect(addr)?;
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

        let msg = &mut self.buf[0..n_read];
        let msg = SensorMessage::<50>::decode(msg).map_err(SubscribeError::DecodeFailed)?;

        self.values = msg.values.to_vec().into_iter();
        Ok(self
            .values
            .next()
            .expect("min values in sensormessage is one"))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SubscribeError {
    #[error("The connection to the subscribe server failed")]
    ConnFailed(std::io::Error),
    #[error("The connection to the subscribe server failed")]
    DecodeFailed(DecodeError),
}
