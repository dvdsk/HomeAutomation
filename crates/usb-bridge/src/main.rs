use std::net::SocketAddr;

use clap::Parser;

use color_eyre::eyre::Context;
use color_eyre::Section;
use data_server::api::data_source::reconnecting;
use tracing::debug;

mod usb;

#[derive(Parser)]
#[command(name = "usb-bridge")]
#[command(version = "1.0")]
#[command(about = "forwards sensor info from and affector orders to nodes attached to usb")]
struct Cli {
    /// Where to send the data on the local system
    #[arg(short, long("data-server"), default_value = "192.168.1.43:1234")]
    data_server: SocketAddr,
    /// Serial number of the device to connect case insensitive
    #[arg(short, long)]
    serial_number: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install().unwrap();
    logger::tracing::setup();

    let args = Cli::parse();
    debug!("Started usb-bridge");

    let (order_tx, order_rx) = tokio::sync::mpsc::channel(10);
    let mut usb = usb::ReconnectingUsb::new(args.serial_number, order_rx);
    let affectors = usb
        .get_affectors()
        .await
        .wrap_err("Could not get affector list")?;
    dbg!(&affectors);
    let mut server_client = reconnecting::Client::new(
        args.data_server,
        affectors,
        Some(order_tx),
    ).await?;

    loop {
        let encoded_msg = usb.handle_usb().await;
        server_client
            .check_send_encoded(encoded_msg)
            .await
            .wrap_err("Should be correctly encoded")
            .suggestion("Check if this needs to be updated")?;
    }
}
