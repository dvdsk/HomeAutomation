use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::sync::mpsc;
use std::thread;

use clap::Parser;
use color_eyre::eyre::WrapErr;
use color_eyre::{Help, Result};
use data_server::SubMessage;

mod fetch;
mod populate;
mod time;
mod tui;

pub(crate) use fetch::Fetch;
use log_store::api::{ErrorEvent, Percentile};
use protocol::Reading;

#[derive(Debug, PartialEq, Eq)]
enum UserIntent {
    Shutdown,
}

struct Fetched {
    reading: Reading,
    thing: Fetchable
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

enum Update {
    ReadingList(Vec<protocol::Reading>),
    Fetched{reading: Reading, thing: Fetchable},
    FetchError(color_eyre::Report),
    SensorReading(protocol::Reading),
    SensorError(Box<protocol::Error>),
    SubscribeError(color_eyre::Report),
    DeviceList(Vec<protocol::Device>),
}

fn receive_data(mut sub: data_server::Subscriber, tx: mpsc::Sender<Update>) {
    loop {
        let res = sub
            .next_msg()
            .wrap_err("Error getting next reading from server")
            .map(|msg| match msg {
                SubMessage::Reading(reading) => Update::SensorReading(reading),
                SubMessage::ErrorReport(error) => Update::SensorError(error),
            });

        match res {
            Ok(msg) => {
                tx.send(msg).unwrap();
            }
            Err(err) => {
                tx.send(Update::SubscribeError(err)).unwrap();
                break;
            }
        }
    }
}

#[derive(Parser)]
#[command(name = "sensor tui")]
#[command(version = "1.0")]
#[command(about = "View sensor values")]
struct Cli {
    /// server where we can subscribe for sensor data updates
    #[arg(short, long, default_value_t = SocketAddr::from(([192,168,1,43], 1235)))]
    data_server: SocketAddr,

    /// server where we can fetch historical sensor data
    #[arg(short='s', long, default_value_t = SocketAddr::from(([192,168,1,43], 1236)))]
    data_store: SocketAddr,

    /// server where we can fetch logs and timing information
    #[arg(short, long, default_value_t = SocketAddr::from(([192,168,1,43], 1237)))]
    log_store: SocketAddr,
}

fn main() -> Result<()> {
    setup_tracing().unwrap();

    let Cli {
        data_server,
        data_store,
        log_store,
    } = Cli::parse();

    let sub = data_server::Subscriber::connect(data_server, &client_name())
        .wrap_err("failed to connect")
        .with_suggestion(|| format!("verify the server is listening on: {data_server}"))?;

    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();

    let (fetcher, maintain_fetch) = Fetch::new(data_store, log_store, tx1.clone());

    let tx1_clone1 = tx1.clone();
    let tx1_clone2 = tx1.clone();
    thread::spawn(move || receive_data(sub, tx1_clone1));
    thread::spawn(move || tui::run(rx1, tx2, fetcher));
    thread::spawn(move || maintain_fetch);
    thread::spawn(move || populate::tree(data_store, log_store, tx1_clone2));

    loop {
        let UserIntent::Shutdown = rx2.recv()?;
        break Ok(());
    }
}

fn setup_tracing() -> Result<()> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, Layer};

    color_eyre::install().unwrap();

    let log_file = std::fs::File::create("log.txt")?;
    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_writer(log_file)
        .with_target(false)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .init();
    Ok(())
}

fn client_name() -> String {
    let host = gethostname::gethostname();
    let host = host.to_string_lossy();
    format!("sensor-tui@{host}")
}

/// Similar to the `std::dbg!` macro, but generates `tracing` events rather
/// than printing to stdout.
///
/// By default, the verbosity level for the generated events is `DEBUG`, but
/// this can be customized.
#[macro_export]
macro_rules! trace_dbg {
    (target: $target:expr, level: $level:expr, $ex:expr) => {{
        match $ex {
            value => {
                tracing::event!(target: $target, $level, ?value, stringify!($ex));
                value
            }
        }
    }};
    (level: $level:expr, $ex:expr) => {
        trace_dbg!(target: module_path!(), level: $level, $ex)
    };
    (target: $target:expr, $ex:expr) => {
        trace_dbg!(target: $target, level: tracing::Level::DEBUG, $ex)
    };
    ($ex:expr) => {
        trace_dbg!(level: tracing::Level::DEBUG, $ex)
    };
}
