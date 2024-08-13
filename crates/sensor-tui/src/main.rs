use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;

use clap::Parser;
use color_eyre::eyre::WrapErr;
use color_eyre::{Help, Result};
use data_server::SubMessage;

mod time;
mod tui;
mod populate;

enum Update {
    ReadingList(Vec<protocol::Reading>),
    Reading(protocol::Reading),
    Error(Box<protocol::Error>),
    Shutdown,
    DeviceList(Vec<protocol::Device>),
}

fn receive_data(mut sub: data_server::Subscriber, tx: mpsc::Sender<Result<Update>>) {
    loop {
        let update = sub
            .next_msg()
            .wrap_err("Error getting next reading from server")
            .map(|msg| match msg {
                SubMessage::Reading(reading) => Update::Reading(reading),
                SubMessage::ErrorReport(error) => Update::Error(error),
            });

        let err = update.is_err();
        tx.send(update).unwrap();
        if err {
            break;
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

    let tx1_clone = tx1.clone();
    let tx1_clone_clone = tx1.clone();
    thread::spawn(move || receive_data(sub, tx1_clone));
    thread::spawn(move || populate::tree(data_store, log_store, tx1_clone_clone));
    thread::spawn(move || tui::run(rx2, tx1, data_store, log_store));

    loop {
        let update = rx1.recv()??;
        if let Update::Shutdown = update {
            break Ok(());
        };
        tx2.send(update).unwrap()
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
