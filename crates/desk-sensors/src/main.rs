use clap::Parser;
use color_eyre::Result;
use data_server::api::data_source;
use std::net::SocketAddr;

use tokio::sync::mpsc;

use tracing::info;

mod bedroom;
mod sensors;

#[derive(Parser)]
#[command(name = "local sensors")]
#[command(version = "1.0")]
#[command(about = "reads sensors attached to rpi gpio pins and i2c perhipheral")]
struct Cli {
    /// Where to send the data on the local system
    #[arg(short, long("data-server"))]
    data_server: SocketAddr,
    /// Is this the pi in the large bedroom or small bedroom?
    #[arg(short, long)]
    bedroom: bedroom::Bedroom,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    color_eyre::install().unwrap();
    let cli = Cli::parse();

    setup_tracing().unwrap();

    let (tx, mut rx) = mpsc::channel(100);
    sensors::start_monitoring(tx.clone(), cli.bedroom);

    info!("connecting to dataserver on: {}", cli.data_server);
    let mut client = data_source::reconnecting::Client::new(cli.data_server, Vec::new());

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
