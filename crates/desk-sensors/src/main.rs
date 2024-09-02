use clap::Parser;
use color_eyre::Result;
use protocol::{Msg, SensorMessage};
use std::io::{ErrorKind, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::time::Duration;
use tracing::{debug, info, warn};

mod sensors;

#[derive(Parser)]
#[command(name = "local sensors")]
#[command(version = "1.0")]
#[command(about = "reads sensors attached to rpi gpio pins and i2c perhipheral")]
struct Cli {
    /// where to send the data on the local system
    #[arg(short, long, default_value = "1234")]
    update_port: u16,
}

fn main() {
    color_eyre::install().unwrap();
    let cli = Cli::parse();

    setup_tracing().unwrap();

    let (tx, rx) = mpsc::channel();
    if let Err(error) = sensors::start_monitoring(tx.clone()) {
        send_error(&tx, error);
    }

    let addr = SocketAddr::from(([127, 0, 0, 1], cli.update_port));
    info!("connecting to dataserver on: {}", cli.update_port);

    loop {
        let mut stream = match TcpStream::connect(addr) {
            Ok(stream) => stream,
            Err(err) if err.kind() == ErrorKind::ConnectionRefused => {
                warn!("could not connect to data server, retrying in 5 seconds");
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
            Err(err) => panic!("could not connect to server: {}", err),
        };

        let bytes =
            protocol::Msg::AffectorList(protocol::affector::ListMessage::<50>::empty()).encode();
        if stream.write_all(&bytes).is_err() {
            std::thread::sleep(Duration::from_secs(5));
            break; // reconnect
        }

        loop {
            let result = rx.recv().unwrap();
            let msg = match result {
                Ok(reading) => {
                    let mut readings = SensorMessage::<1>::default();
                    readings
                        .values
                        .push(reading)
                        .expect("capacity allows one push");
                    Msg::Readings(readings)
                }
                Err(report) => Msg::ErrorReport(protocol::ErrorReport::new(report)),
            };

            debug!("Sending message: {msg:?}");
            let bytes = msg.encode();
            if stream.write_all(&bytes).is_err() {
                std::thread::sleep(Duration::from_secs(5));
                break; // reconnect
            }
        }
    }
}

fn setup_tracing() -> Result<()> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, Layer};

    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .init();
    Ok(())
}

fn send_error(
    tx: &Sender<Result<protocol::Reading, protocol::Error>>,
    error: protocol::large_bedroom::desk::Error,
) {
    use protocol::large_bedroom::Error::Desk as DeskE;
    use protocol::Error::LargeBedroom as LbE;
    tx.send(Err(LbE(DeskE(error)))).unwrap();
}

fn send_reading(
    tx: &Sender<Result<protocol::Reading, protocol::Error>>,
    reading: protocol::large_bedroom::desk::Reading,
) {
    use protocol::large_bedroom::Reading::Desk;
    use protocol::Reading::LargeBedroom as Lb;
    tx.send(Ok(Lb(Desk(reading)))).unwrap();
}
