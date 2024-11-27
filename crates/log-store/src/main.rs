use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::Result;

#[derive(Parser, Debug)]
#[command(name = "data server")]
#[command(version = "1.0")]
#[command(about = "Receives sensor events then logs errors and tracks timing")]
struct Cli {
    /// data server
    #[arg(short, long)]
    data_server: SocketAddr,

    #[arg(short, long)]
    client_port: u16,

    #[arg(long, default_value = ".")]
    log_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    logger::tracing::setup();
    let cli = Cli::parse();
    tracing::info!("started log-store, args: {cli:?}");

    log_store::server::run(cli.data_server, cli.client_port, &cli.log_dir).await
}
