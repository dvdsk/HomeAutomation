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
    /// Addr to which subscribers can connect
    #[arg(short, long)]
    subscribe_addr: SocketAddr,

    /// Addr to which data-source can supply messages
    #[arg(short, long)]
    update_addr: SocketAddr,

    /// Reset data-sources that have missing or slow sensors
    #[arg(short, long)]
    enable_reset: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    logger::tracing::setup();

    let Cli {
        subscribe_addr,
        update_addr,
        enable_reset,
    } = Cli::parse();
    assert_ne!(subscribe_addr, update_addr);

    info!("listening for updates on: {update_addr}");
    info!("serving subscribers on: {subscribe_addr}");

    let affectors = server::AffectorRegistar::default();
    let (tx, rx) = mpsc::channel(2000);

    if enable_reset {
        select! {
            e = server::client::handle(subscribe_addr, tx.clone(), affectors.clone()) => e,
            e = server::handle_nodes(update_addr, &tx, affectors.clone()) => e,
            e = server::spread_updates(rx) => e,
            _ = server::node_watchdog(affectors, &tx) => Ok(()),
        }
    } else {
        select! {
            e = server::client::handle(subscribe_addr, tx.clone(), affectors.clone()) => e,
            e = server::handle_nodes(update_addr, &tx, affectors.clone()) => e,
            e = server::spread_updates(rx) => e,
        }
    }
}
