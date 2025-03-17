use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;
use color_eyre::eyre::Result;

#[derive(Parser, Debug)]
#[command(name = "data server")]
#[command(version = "1.0")]
#[command(
    about = "Receives sensor events and spreads those to subscribed services"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    #[arg(long, default_value = ".")]
    data_dir: PathBuf,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    Run {
        /// data server
        #[arg(short, long)]
        data_server: SocketAddr,

        #[arg(short, long)]
        client_port: u16,
    },
    Export {
        /// export only one dataset at this path
        /// `largebedroom/bed/nau7802right`
        #[arg(short, long)]
        only: Option<PathBuf>,
    },
    Import {
        /// import a dataset at this path for example:
        /// `largebedroom/bed/nau7802right`
        #[arg(short, long)]
        only: Option<PathBuf>,
        /// skip data that is out of order, this is useful as it fixes an
        /// issue caused by a bug in Byteseries.
        skip_corrupt: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    logger::tracing::setup();
    color_eyre::install().unwrap();
    let cli = Cli::parse();
    tracing::info!("started data-server, args: {cli:?}");

    match cli.command {
        Command::Run {
            data_server,
            client_port,
        } => {
            data_store::server::run(data_server, client_port, &cli.data_dir)
                .await
        }
        Command::Export { only } => {
            data_store::export::perform(&cli.data_dir, only)
        }
        Command::Import { only, skip_corrupt } => {
            data_store::import::perform(&cli.data_dir, only, skip_corrupt)
        }
    }
}
