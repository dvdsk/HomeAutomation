use std::net::SocketAddr;

use clap::Parser;

use color_eyre::eyre::Context;
use color_eyre::Section;
use data_server::api::data_source::reconnecting;

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
    setup_tracing();
    let args = Cli::parse();

    let (order_tx, order_rx) = tokio::sync::mpsc::channel(10);
    let mut usb = usb::ReconnectingUsb::new(args.serial_number, order_rx);
    let affectors = usb
        .get_affectors()
        .await
        .wrap_err("Could not get affector list")?;
    let mut server_client = reconnecting::Client::new(args.data_server, affectors, Some(order_tx));

    loop {
        let encoded_msg = usb.handle_usb().await;
        server_client
            .check_send_encoded(&encoded_msg)
            .await
            .wrap_err("Should be correctly encoded")
            .suggestion("Check if this needs to be updated")?;
    }
}

fn setup_tracing() {
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
}
