use std::time::Duration;

use protocol::Reading;

use serde::{Deserialize, Serialize};

pub mod client;
pub use client::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Request {
    Handshake {
        name: String,
    },
    ListData,
    GetData {
        reading: Reading,
        start: jiff::Timestamp,
        end: jiff::Timestamp,
        n: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum ServerError {
    #[error("We do not have any data for this reading: {reading:?}")]
    NotInStore { reading: Reading },
    #[error("Internal error while reading data, error: {0}")]
    ReadingFromStore(String),
    #[error("Connect request should only be send once")]
    AlreadyConnected,
    #[error("Too many requests, rate limited, next requested allowed in: {0:?}")]
    TooManyRequests(Duration),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Response {
    ListData(Vec<Reading>),
    GetData {
        time: Vec<jiff::Timestamp>,
        data: Vec<f32>,
    },
    Error(ServerError),
    Handshake,
}
