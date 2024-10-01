pub mod control;
mod fetch;
pub mod populate;
pub mod receive;
mod time;
pub mod tui;

pub use fetch::Fetch;
use log_store::api::{ErrorEvent, Percentile};
use protocol::Reading;
use std::ops::RangeInclusive;

fn client_name() -> String {
    let host = gethostname::gethostname();
    let host = host.to_string_lossy();
    format!("sensor-tui@{host}")
}

enum Fetchable {
    Data {
        timestamps: Vec<jiff::Timestamp>,
        data: Vec<f32>,
    },
    Logs {
        logs: Vec<ErrorEvent>,
        start_at: jiff::Timestamp,
    },
    Hist {
        percentiles: Vec<Percentile>,
        range: RangeInclusive<jiff::Timestamp>,
    },
}

pub enum Update {
    ReadingList(Vec<protocol::Reading>),
    Fetched {
        reading: Reading,
        thing: Fetchable,
    },
    FetchError(color_eyre::Report),
    SensorReading(protocol::Reading),
    SensorError(Box<protocol::Error>),
    SubscribeError(color_eyre::Report),
    DeviceList(Vec<protocol::Device>),
    AffectorControlled {
        affector: protocol::Affector,
        controlled_by: String,
    },
    AffectorList(Vec<protocol::Affector>),
    PopulateError(color_eyre::Report),
}

#[derive(Debug, PartialEq, Eq)]
pub enum UserIntent {
    Shutdown,
}
