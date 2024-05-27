use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;

use color_eyre::Result;

mod tui;

enum Update {
    Reading(protocol::Reading),
    Error(protocol::Error),
}

fn main() -> Result<()> {
    setup_tracing().unwrap();
    let addr = SocketAddr::from(([127, 0, 0, 1], 1235));
    let mut sub = data_server::Subscriber::connect(addr).unwrap();

    let (tx, rx) = mpsc::channel();
    thread::spawn(|| tui::run(rx));

    while let Ok(val) = sub.next() {
        let update = match val {
            Ok(reading) => Update::Reading(reading),
            Err(error) => Update::Error(error),
        };

        tx.send(update).unwrap();
    }

    Ok(())
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
