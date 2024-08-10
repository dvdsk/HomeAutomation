use std::time::Duration;

use protocol::Reading;

use serde::{Deserialize, Serialize};

pub mod client;
pub use client::Client;

// 8 MB
pub(crate) const MAX_PACKAGE_SIZE: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Request {
    Handshake { name: String },
    GetLog(protocol::Device),
    GetStats(protocol::Device),
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum ServerError {
    #[error("Connect request should only be send once")]
    AlreadyConnected,
    #[error("Too many requests, rate limited, next requested allowed in: {0:?}")]
    TooManyRequests(Duration),
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum GetStatsError {
    #[error("We do not have any data for this reading: {reading:?}")]
    NotInStore { reading: Reading },
    #[error("Internal error while reading data, error: {0}")]
    InternalError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum GetLogError {
    #[error("Too many error events ({found}) between requested timepoints, max is: {max}")]
    TooMuchData { max: i32, found: u64 },
    #[error("Internal error while reading data, error: {0}")]
    InternalError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Percentile {
    pub bucket_ends: u64,
    pub percentile: f64,
    pub count_in_bucket: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub start: jiff::Timestamp,
    pub end: jiff::Timestamp,
    pub error: protocol::Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Response {
    GetLog(Result<Vec<ErrorEvent>, GetLogError>),
    GetStats(Result<Vec<Percentile>, GetStatsError>),
    Error(ServerError),
    Handshake,
}
