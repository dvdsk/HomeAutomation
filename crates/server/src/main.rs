use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;

use color_eyre::Result;

use protocol::SensorMessage;

mod tui;

fn handle_client(stream: TcpStream, tx: Sender<Update>) {
    let mut reader = BufReader::new(stream);
    let mut buf = Vec::new();
    loop {
        let n_read = match reader.read_until(0, &mut buf) {
            Err(e) => {
                tx.send(Update::ConnFailed).unwrap();
                return;
            }
            Ok(bytes) => bytes,
        };
        let msg = &mut buf[0..n_read];
        let msg = match SensorMessage::<6>::decode(msg) {
            Ok(msg) => msg,
            Err(e) => {
                tx.send(Update::DecodeFailed(e)).unwrap();
                continue;
            }
        };
        let values = msg.values;
        for value in values.into_iter() {
            tx.send(Update::Data(value)).unwrap();
        }

        buf.clear();
    }
}

enum Update {
    Data(protocol::Sensor),
    ConnFailed,
    NewConn(std::net::SocketAddr),
    DecodeFailed(protocol::DecodeError),
}

fn main() -> Result<()> {
    setup_tracing().unwrap();
    let (tx, rx) = mpsc::channel();

    thread::spawn(|| tui::run(rx));

    let listener = TcpListener::bind("0.0.0.0:1234")?;

    // accept connections and process them serially
    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(e) => {
                println!("new connection failed: {e}");
                continue;
            }
        };

        tx.send(Update::NewConn(stream.peer_addr().unwrap()))
            .unwrap();
        {
            let tx = tx.clone();
            std::thread::spawn(move || handle_client(stream, tx));
        }
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
