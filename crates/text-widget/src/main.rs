use color_eyre::eyre::{eyre, Context};
use color_eyre::Result;
use data_server::api::SubMessage;
use data_server::api::{ReconnectingClient, ReconnectingSubscribedClient};
use gethostname::gethostname;
use protocol::reading::tree::Tree;
use protocol::Reading;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::timeout;

use clap::Parser;

mod cache;
mod resolve;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// the address of the store, optional, speeds up resolving the argument
    #[arg(long)]
    store: Option<SocketAddr>,

    /// the address of the data server
    #[arg(long, default_value_t = SocketAddr::from(([127,0,0,1], 1235)))]
    server: SocketAddr,

    /// print json format: {"msg": "reading value"}
    #[arg(short, long)]
    json: bool,

    /// String describing the reading. Something like temp can resolve to
    /// `large_bedroom desk temperature`
    reading: String,

    /// prints the resolved string
    #[arg(short, long)]
    debug: bool,
}

async fn wait_for_update(client: &mut ReconnectingSubscribedClient, needed: &Reading) -> Reading {
    loop {
        let SubMessage::Reading(r) = client.next().await else {
            continue;
        };
        if r.is_same_as(needed) {
            return r;
        }
    }
}

async fn setup(cli: &Cli, client: &mut ReconnectingSubscribedClient) -> Result<protocol::Reading> {
    let reading = match resolve::query(cli, client).await {
        Ok(reading) => reading,
        Err(e) => {
            print(cli.json, "E");
            return Err(e);
        }
    };
    if promptly::prompt_default(format!("Is {reading:?} the correct sensor?"), false)
        .wrap_err("Failed to read user confirmation")?
    {
        cache::store_to_file(reading.clone(), cli.reading.clone()).await?;
        Ok(reading)
    } else {
        Err(eyre!("Exited, user indicated resolved sensor is incorrect"))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    setup_tracing(cli.debug)?;

    let mut client = ReconnectingClient::new(cli.server, name()).subscribe();

    let reading = match cache::load_from_file(&cli.reading).await {
        Ok(Some(reading)) => reading,
        Ok(None) => {
            print(cli.json, "RunSetup");
            setup(&cli, &mut client).await?
        }
        Err(e) => {
            print(cli.json, "Error");
            return Err(eyre!("Error, could not load resolved from file: {e:?}"));
        }
    };

    tracing::debug!("Will be showing: {reading:?}");
    let timeout_dur = reading.leaf().device.info().max_sample_interval + Duration::from_secs(1);

    loop {
        let get_update = wait_for_update(&mut client, &reading);
        let Ok(new) = timeout(timeout_dur, get_update).await else {
            print(cli.json, "X");
            continue;
        };

        let info = new.leaf();
        let to_print = format!("{0:.1$} {2}", info.val, info.precision(), info.unit);
        print(cli.json, &to_print);
    }
}

fn name() -> String {
    let host = gethostname();
    let host = host.to_string_lossy();
    format!("text-widget@{host}")
}

fn print(use_json: bool, msg: &str) {
    if use_json {
        println!("{{\"msg\": \"{msg}\"}}");
    } else {
        println!("{msg}");
    }
}

fn setup_tracing(debug: bool) -> Result<()> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::filter;
    use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

    color_eyre::install().unwrap();

    let filter = filter::EnvFilter::builder().from_env().unwrap();
    let filter = if debug {
        let directive = concat!(env!("CARGO_PKG_NAME"), "=debug").parse()?;
        filter.add_directive(directive)
    } else {
        filter
    };

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
