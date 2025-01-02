use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::Result;

#[derive(Parser, Debug)]
#[command(name = "data server")]
#[command(version = "1.0")]
#[command(about = "Receives sensor events and spreads those to subscribed services")]
struct Cli {
    /// data server
    #[arg(short, long)]
    data_server: SocketAddr,

    #[arg(short, long)]
    client_port: u16,

    #[arg(long, default_value = ".")]
    data_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    logger::tracing::setup();
    color_eyre::install().unwrap();
    let cli = Cli::parse();
    tracing::info!("started data-server, args: {cli:?}");

    data_store::server::run(cli.data_server, cli.client_port, &cli.data_dir).await
}
