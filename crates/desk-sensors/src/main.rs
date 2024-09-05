use clap::Parser;
use color_eyre::Result;
use data_server::api::data_source;
use std::net::SocketAddr;

use tokio::sync::mpsc::{self, Sender};

use tracing::info;

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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    color_eyre::install().unwrap();
    let cli = Cli::parse();

    setup_tracing().unwrap();

    let (tx, mut rx) = mpsc::channel(100);
    if let Err(error) = sensors::start_monitoring(tx.clone()) {
        send_error(&tx, error);
    }

    let addr = SocketAddr::from(([127, 0, 0, 1], cli.update_port));
    info!("connecting to dataserver on: {}", cli.update_port);
    let mut client = data_source::reconnecting::Client::new(addr, Vec::new());

    loop {
        match rx.recv().await.expect("sensor monitoring never stops") {
            Ok(reading) => client.send_reading(reading).await,
            Err(report) => client.send_error(report).await,
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
    tx.blocking_send(Err(LbE(DeskE(error)))).unwrap();
}

fn send_reading(
    tx: &Sender<Result<protocol::Reading, protocol::Error>>,
    reading: protocol::large_bedroom::desk::Reading,
) {
    use protocol::large_bedroom::Reading::Desk;
    use protocol::Reading::LargeBedroom as Lb;
    tx.blocking_send(Ok(Lb(Desk(reading)))).unwrap();
}
