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
    setup_tracing().unwrap();
    let cli = Cli::parse();
    tracing::info!("started log-store, args: {cli:?}");

    log_store::server::run(cli.data_server, cli.client_port, &cli.log_dir).await
}

fn setup_tracing() -> Result<()> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::filter;
    use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

    color_eyre::install().unwrap();

    let filter = filter::EnvFilter::builder().from_env().unwrap();
    let fmt = tracing_subscriber::fmt::layer()
        .pretty()
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt)
        .with(ErrorLayer::default())
        .init();
    Ok(())
}
