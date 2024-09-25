use std::array;
use std::net::{IpAddr, SocketAddr};

use clap::Parser;
use tokio::sync::broadcast;
use tracing::warn;

mod controller;
mod system;

mod errors;
mod input;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Opt {
    /// ip address whete to subscribe for updates
    #[clap(short, long)]
    data_server: SocketAddr,

    /// ip address for mpd server
    #[clap(short, long)]
    mpd_ip: IpAddr,

    /// http api listens on this port at 127.0.0.1 use
    /// a loadbalancer/reverse proxy to get traffic to this
    #[clap(short, long)]
    port: u16,

    /// ip address of the hue bridge, format as: 127.0.0.1
    #[clap(short = 'b', long)]
    hue_bridge_ip: String,
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

    // must create all listeners before jobs/alarm events can be send
    // or they may be lost.
    let (event_tx, event_rx) = broadcast::channel(250);
    let subscribed_rxs = array::from_fn(|_| event_tx.subscribe());

    let (jobs, _waker_thread) = input::jobs::Jobs::setup(event_tx.clone(), db.clone())?;
    let wakeup = input::jobs::WakeUp::setup(db.clone(), jobs.clone(), event_rx)?;
    // let (_mpd_status, _mpd_watcher_thread, _updater_tx) =
    //     input::MpdStatus::start_updating(opt.mpd_ip)?;

    let system = system::System::init(jobs, opt.hue_bridge_ip);
    let _tasks = controller::start(subscribed_rxs, event_tx.clone(), system);

    tokio::task::spawn(input::sensors::subscribe(event_tx, opt.data_server));
    input::api::setup(wakeup.clone(), opt.port).await?;

    unreachable!();
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
                .parse("brain=debug,info")
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
}
