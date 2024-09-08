use std::time::Duration;

use protocol::Reading;

use serde::{Deserialize, Serialize};

pub mod client;
pub use client::Client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Request {
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
pub struct Data {
    pub time: Vec<jiff::Timestamp>,
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Response {
    ListData(Vec<Reading>),
    GetData(Result<Data, GetDataError>),
    Error(ServerError),
    Handshake,
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum GetDataError {
    #[error("could not find timestamp in this series")]
    NotFound,
    #[error("data file is empty")]
    EmptyFile,
    #[error("no data to return as the start time is after the last time in the data")]
    StartAfterData,
    #[error("no data to return as the stop time is before the data")]
    StopBeforeData,
    #[error("We do not have any data for this reading: {reading:?}")]
    NotInStore { reading: Reading },
    #[error("Internal error while reading data, error: {0}")]
    ReadingFromStore(String),
}
