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
    #[arg(short, long, default_value = "1234")]
    update_port: u16,
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
