use std::net::IpAddr;

use clap::Parser;
use sled;
use tokio::sync::broadcast;
use tracing::warn;

mod controller;
mod system;

mod errors;
mod input;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Opt {
    /// secret url part on which sensor data is received
    #[clap(short, long)]
    key: String,

    /// ip address for mpd server
    #[clap(short, long)]
    mpd_ip: IpAddr,

    /// http api listens on this port at 127.0.0.1 use
    /// a loadbalancer/reverse proxy to get traffic to this
    #[clap(short, long)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_tracing();
    let opt = Opt::parse();

    let db = sled::Config::default() //651ms
        .path("database")
        .flush_every_ms(None) //do not flush to disk unless explicitly asked
        .cache_capacity(1024 * 1024 * 32) //32 mb cache
        .open()?;

    let (event_tx, _event_rx) = broadcast::channel(250);

    let (jobs, _waker_thread) = input::jobs::Jobs::setup(event_tx.clone(), db.clone())?;
    let wakeup = input::jobs::WakeUp::setup(db.clone(), jobs.clone())?;
    // let (_mpd_status, _mpd_watcher_thread, _updater_tx) =
    //     input::MpdStatus::start_updating(opt.mpd_ip)?;

    let system = system::System::init(jobs);
    controller::start(event_tx.clone(), system);
    input::api::setup(wakeup.clone(), event_tx, opt.port).await?;

    std::future::pending::<()>().await;
    Ok(())
}

pub fn setup_tracing() {
    use tracing_subscriber::filter;
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;

    let filter = filter::EnvFilter::builder()
        .with_regex(true)
        .try_from_env()
        .unwrap_or_else(|_| {
            filter::EnvFilter::builder()
                .parse("HomeAutomation=debug,info")
                .unwrap()
        });

    let fmt = fmt::layer()
        .pretty()
        .with_line_number(true)
        .with_test_writer();

    let registry = tracing_subscriber::registry().with(filter).with(fmt);
    match tracing_journald::layer() {
        Ok(journal) => registry.with(journal).init(),
        Err(err) => {
            warn!("Failed to init journald logging, error: {err}");
            registry.init()
        }
    };
    log_panics::init();
}
