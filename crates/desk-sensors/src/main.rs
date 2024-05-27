use clap::Parser;
use color_eyre::Result;
use std::io::{ErrorKind, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::time::Duration;

mod buttons;
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
    buttons::start_monitoring(tx.clone());
    if let Err(error) = sensors::start_monitoring(tx.clone()) {
        send_error(&tx, error);
    }

    let mut msg: protocol::SensorMessage<20> = protocol::SensorMessage::new();
    let addr = SocketAddr::from(([127, 0, 0, 1], cli.update_port));
    loop {
        let mut stream = match TcpStream::connect(addr) {
            Ok(stream) => stream,
            Err(err) if err.kind() == ErrorKind::ConnectionRefused => {
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
            Err(err) => panic!("could not connect to server: {}", err),
        };

        loop {
            let result = rx.recv().unwrap();
            msg.values.push(result).expect("capacity > 0");
            while let Ok(result) = rx.try_recv() {
                let res = msg.values.push(result);
                if res.is_err() {
                    break;
                }
            }
            let bytes = msg.encode();
            stream.write_all(&bytes).unwrap();
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
