use protocol::Reading;

use serde::{Serialize, Deserialize};
use time::OffsetDateTime;

mod client;
pub use client::Client;

// 8 MB
pub(crate) const MAX_PACKAGE_SIZE: usize = 8*1024*1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Request {
    ListData,
    GetData {
        reading: Reading,
        start: OffsetDateTime,
        end: OffsetDateTime,
        n: usize,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum ServerError {
    #[error("We do not have any data for this reading: {reading:?}")]
    NotInStore { reading: Reading },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Response {
    ListData(Vec<Reading>),
    GetData(Vec<(OffsetDateTime, f32)>),
    Error(ServerError),
}
