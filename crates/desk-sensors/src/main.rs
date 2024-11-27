use clap::Parser;
use color_eyre::eyre::Context;
use data_server::api::data_source::reconnecting::Client;
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

    logger::tracing::setup();

    let (tx, mut rx) = mpsc::channel(100);
    sensors::start_monitoring(tx.clone(), cli.bedroom);

    info!("connecting to dataserver on: {}", cli.data_server);
    let mut client = Client::new(cli.data_server, Vec::new(), None)
        .await
        .unwrap();

    loop {
        let res = match rx.recv().await.expect("sensor monitoring never stops") {
            Ok(reading) => client
                .send_reading(reading)
                .await
                .wrap_err("Sending reading"),
            Err(report) => client
                .send_error(report)
                .await
                .wrap_err("Sending error report"),
        };

        if let Err(e) = res {
            tracing::warn!("{e}");
        }
    }
}
