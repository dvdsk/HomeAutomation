use protocol::Reading;

use serde::{Deserialize, Serialize};

mod client;
pub use client::Client;

// 8 MB
pub(crate) const MAX_PACKAGE_SIZE: usize = 8 * 1024 * 1024;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Response {
    ListData(Vec<Reading>),
    GetData {
        time: Vec<jiff::Timestamp>,
        data: Vec<f32>,
    },
    Error(ServerError),
}
