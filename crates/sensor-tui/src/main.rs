use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;

use clap::Parser;
use color_eyre::Result;
use sensor_tui::control;
use sensor_tui::UserIntent;
use tokio::task;

use sensor_tui::Fetch;
use sensor_tui::tui;
use sensor_tui::receive;
use sensor_tui::populate;


#[derive(Parser)]
#[command(name = "sensor tui")]
#[command(version = "1.0")]
#[command(about = "View sensor values")]
struct Cli {
    /// Server where we can subscribe for sensor data updates
    #[arg(short, long, default_value_t = SocketAddr::from(([192,168,1,43], 1235)))]
    data_server: SocketAddr,

    /// Server where we can fetch historical sensor data
    #[arg(short='s', long, default_value_t = SocketAddr::from(([192,168,1,43], 1236)))]
    data_store: SocketAddr,

    /// Server where we can fetch logs and timing information
    #[arg(short, long, default_value_t = SocketAddr::from(([192,168,1,43], 1237)))]
    log_store: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    logger::tracing::setup();

    let Cli {
        data_server,
        data_store,
        log_store,
    } = Cli::parse();

    let (tx1, rx1) = mpsc::channel();
    let (tx2, rx2) = mpsc::channel();
    let (tx3, rx3) = tokio::sync::mpsc::channel(100);

    let fetcher = Fetch::new(data_store, log_store, tx1.clone());

    let tx1_clone1 = tx1.clone();
    let tx1_clone2 = tx1.clone();

    thread::spawn(move || tui::run(rx1, tx2, tx3, fetcher));
    task::spawn(receive::receive_data(data_server, tx1_clone1));
    task::spawn(populate::tree(
        data_server,
        data_store,
        log_store,
        tx1_clone2,
    ));
    task::spawn(control::watch_and_send(data_server, rx3));

    let UserIntent::Shutdown = rx2.recv()?;
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
