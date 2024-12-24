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

const JOBS_DB_PATH: &str = "database/jobs";

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Opt {
    /// IP address where to subscribe for updates
    #[clap(long)]
    data_server: SocketAddr,

    /// IP address for mpd server
    #[clap(long)]
    mpd_ip: IpAddr,

    /// http API listens on this port at 127.0.0.1 use
    /// a loadbalancer/reverse proxy to get traffic to this
    #[clap(long)]
    http_port: u16,

    /// IP address for MQTT broker
    #[clap(long)]
    mqtt_ip: IpAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install().unwrap();
    logger::tracing::setup();
    let opt = Opt::parse();

    // must create all listeners before jobs/alarm events can be send
    // or they may be lost.
    let (event_tx, _event_rx) = broadcast::channel(250);
    let subscribed_rxs = array::from_fn(|_| event_tx.subscribe());

    let jobs = Jobs::setup(event_tx.clone(), JOBS_DB_PATH)?;

    let system = System::init(opt.mqtt_ip, jobs);
    let _tasks = controller::start(subscribed_rxs, event_tx.clone(), system);

    // This never returns, should be replaced by an endless loop if (re)moved
    input::sensors::subscribe(event_tx, opt.data_server).await;

    unreachable!();
}
