use std::array;
use std::net::{IpAddr, SocketAddr};

use clap::Parser;
use tokio::sync::broadcast;

use self::input::jobs::Jobs;
use self::system::System;

mod controller;
mod errors;
mod input;
mod system;
mod time;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Opt {
    /// IP address where to subscribe for updates
    #[clap(short, long)]
    data_server: SocketAddr,

    /// IP address for mpd server
    #[clap(short, long)]
    mpd_ip: IpAddr,

    /// http api listens on this port at 127.0.0.1 use
    /// a loadbalancer/reverse proxy to get traffic to this
    #[clap(short, long)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger::tracing::setup();
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

    let jobs = Jobs::setup(event_tx.clone(), db.clone())?;
    // TODO: untangle the large bedroom wakeup logic from everything else
    let wakeup = controller::large_bedroom::wakeup::WakeUp::setup(
        db.clone(),
        jobs.clone(),
        event_rx,
    )?;

    let system = System::init(jobs);
    let _tasks = controller::start(subscribed_rxs, event_tx.clone(), system);

    tokio::task::spawn(input::sensors::subscribe(event_tx, opt.data_server));
    input::api::setup(wakeup.clone(), opt.port).await?;

    unreachable!();
}
