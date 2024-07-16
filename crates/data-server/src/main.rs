use std::net::SocketAddr;

use tokio::select;
use tokio::sync::mpsc;

use clap::Parser;
use color_eyre::Result;
use tracing::info;

use data_server::server;

#[derive(Parser)]
#[command(name = "data server")]
#[command(version = "1.0")]
#[command(about = "Receives sensor events and spreads those to subscribed services")]
struct Cli {
    /// addr to which subscribers can connect
    #[arg(short, long)]
    subscribe_addr: SocketAddr,

    /// addr to which data-source can supply msg's
    #[arg(short, long)]
    update_addr: SocketAddr,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    setup_tracing();

    let Cli {
        subscribe_addr,
        update_addr,
    } = Cli::parse();
    assert_ne!(subscribe_addr, update_addr);

    info!("listening for updates on: {update_addr}");
    info!("serving subscribers on: {subscribe_addr}");

    let (tx, rx) = mpsc::channel(2000);
    select! {
        e = server::register_subs(subscribe_addr, &tx) => e,
        e = server::handle_data_sources(update_addr, &tx) => e,
        e = server::spread_updates(rx) => e,
    }
}

fn setup_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, Layer};

    color_eyre::install().unwrap();

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
