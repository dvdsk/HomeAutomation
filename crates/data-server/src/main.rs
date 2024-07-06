use tokio::sync::mpsc;
use tokio::select;

use color_eyre::Result;
use tracing::info;
use clap::Parser;

use data_server::server;

#[derive(Parser)]
#[command(name = "data server")]
#[command(version = "1.0")]
#[command(about = "Receives sensor events and spreads those to subscribed services")]
struct Cli {
    /// Optional name to operate on
    #[arg(short, long)]
    subscribe_port: u16,

    /// Sets a custom config file
    #[arg(short, long)]
    update_port: u16,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    setup_tracing().unwrap();

    let Cli {
        subscribe_port,
        update_port,
    } = Cli::parse();
    assert_ne!(subscribe_port, update_port);

    info!("listening for updates on port: {update_port}");
    info!("serving subscribers on port: {subscribe_port}");

    let (tx, rx) = mpsc::channel(2000);
    select! {
        e = server::register_subs(subscribe_port, &tx) => e,
        e = server::handle_data_sources(update_port, &tx) => e,
        e = server::spread_updates(rx) => e,
    }
}

fn setup_tracing() -> Result<()> {
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
    Ok(())
}
