use color_eyre::eyre::{eyre, Context};
use color_eyre::Result;
use data_server::api::subscriber::SubMessage;
use data_server::api::subscriber::{
    ReconnectingClient, ReconnectingSubscribedClient,
};
use gethostname::gethostname;
use itertools::Itertools;
use protocol::reading::tree::Tree;
use protocol::{IsSameAs, Reading};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::{timeout_at, Instant};

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

    /// print json format where for each query passed in there are two key value pairs:
    /// example for the query `temp`:
    /// { "temp": 23.40, "temp-unit": "C" }
    #[arg(short, long)]
    json: bool,

    /// wipe and redo setup
    #[arg(short, long)]
    setup: bool,

    /// print a custom separator between multiple printed values
    #[arg(short = 'd', long, default_value_t = String::from(" "))]
    separator: String,

    /// String describing the reading. Something like temp can resolve to
    /// `large_bedroom desk temperature`.
    ///
    /// Run with RUST_LOG=debug to print the resolved string
    readings: Vec<String>,
}

type Index = usize;
async fn wait_for_update(
    client: &mut ReconnectingSubscribedClient,
    needed: &[Reading],
) -> (Reading, Index) {
    loop {
        let SubMessage::Reading(update) = client.next().await else {
            continue;
        };
        if let Some(idx) =
            needed.iter().position(|watched| update.is_same_as(watched))
        {
            return (update, idx);
        }
    }
}

async fn setup(
    cli: &Cli,
    client: &mut ReconnectingSubscribedClient,
) -> Result<Vec<protocol::Reading>> {
    let available = resolve::available_readings(cli.store, client).await;
    let mut fully_qualified = Vec::new();
    for query in &cli.readings {
        let reading = resolve::query(&available, query)
            .wrap_err("Could not resolve query to reading")?;
        fully_qualified.push(reading);
    }

    cache::store_to_file(&fully_qualified, &cli.readings).await?;
    Ok(fully_qualified)
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install().unwrap();
    let cli = Cli::parse();
    logger::tracing::setup();

    let mut client = ReconnectingClient::new(cli.server, name()).subscribe();
    assert!(
        !cli.readings.is_empty(),
        "must provide at least one sensor to display"
    );

    if cli.setup {
        cache::clear().await.wrap_err("Could not clear cache")?;
    }

    let readings = match cache::load_from_file(&cli.readings).await {
        Ok(readings) if readings.iter().any(Option::is_none) => {
            println!("RunSetup");
            setup(&cli, &mut client).await?
        }
        Ok(readings) => readings.into_iter().map(Option::unwrap).collect(),
        Err(e) => {
            println!("Error");
            return Err(eyre!(
                "Error, could not load resolved from file: {e:?}"
            ));
        }
    };

    tracing::debug!("Will be showing: {readings:?}");
    let mut next_timeout_at =
        tokio::time::Instant::now() + Duration::from_secs(100);

    let mut entries: Vec<_> = readings
        .iter()
        .map(|reading| Entry::new_for(reading))
        .collect();
    loop {
        let get_update = wait_for_update(&mut client, &readings);
        if let Ok((new, idx)) = timeout_at(next_timeout_at, get_update).await {
            entries[idx].curr_value = Some(new);
            entries[idx].last_updated_at = Instant::now();
        }

        for entry in entries.iter_mut().filter(|e| e.curr_value.is_some()) {
            if entry.last_updated_at.elapsed() > entry.timeout_interval {
                entry.curr_value = None;
            }
        }

        if cli.json {
            update_stdout_json(&entries, &cli.readings);
        } else {
            update_stdout(&entries, &cli.separator);
        }

        next_timeout_at = Instant::now()
            + entries
                .iter()
                .filter(|Entry { curr_value, .. }| curr_value.is_some())
                .map(
                    |Entry {
                         last_updated_at,
                         timeout_interval,
                         ..
                     }| {
                        timeout_interval
                            .saturating_sub(last_updated_at.elapsed())
                    },
                )
                .min()
                .unwrap_or(Duration::from_secs(100));
    }
}

#[derive(Debug)]
struct Entry {
    curr_value: Option<Reading>,
    last_updated_at: Instant,
    timeout_interval: Duration,
}

impl Entry {
    fn new_for(reading: &Reading) -> Self {
        Self {
            curr_value: None,
            last_updated_at: Instant::now(),
            timeout_interval: reading.leaf().device.info().max_sample_interval
                + Duration::from_secs(1),
        }
    }

    fn format(&self) -> String {
        if let Some(reading) = &self.curr_value {
            let info = reading.leaf();
            format!("{0:.1$} {2}", info.val, info.precision(), info.unit)
        } else {
            "X".to_string()
        }
    }
}

fn update_stdout_json(entries: &[Entry], used_queries: &[String]) {
    let json_body = used_queries
        .iter()
        .zip(entries)
        .map(|(query, Entry { curr_value, .. })| {
            let value = curr_value
                .as_ref()
                .map(|r| r.leaf())
                .map(|info| format!("{0:.1$}", info.val, info.precision()))
                .unwrap_or("X".to_string());
            let unit = curr_value
                .as_ref()
                .map(|r| r.leaf().unit)
                .unwrap_or(protocol::Unit::None);

            format!("\"{query}\": {}, \"{query}-unit\": \"{}\"", value, unit)
        })
        .join(", ");
    println!("{{ {json_body} }}")
}

fn update_stdout(entries: &[Entry], separator: &str) {
    let mut updates = entries.iter().map(|entry| entry.format());
    let updates: String = updates.join(separator);
    println!("{}", updates)
}

fn name() -> String {
    let host = gethostname();
    let host = host.to_string_lossy();
    format!("text-widget@{host}")
}
