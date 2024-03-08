use sled;
use structopt::StructOpt;
use tracing::warn;

mod controller;

mod errors;
mod input;

#[derive(StructOpt)]
#[structopt(name = "homeautomation")]
struct Opt {
    /// secret url part on which sensor data is received
    #[structopt(short = "h", long = "ha-key")]
    ha_key: String,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    setup_tracing();

    let db = sled::Config::default() //651ms
        .path("database")
        .flush_every_ms(None) //do not flush to disk unless explicitly asked
        .cache_capacity(1024 * 1024 * 32) //32 mb cache
        .open()
        .unwrap();

    let (controller_tx, controller_rx) = crossbeam_channel::unbounded();

    let (joblist, _waker_thread) =
        input::jobs::Jobs::setup(controller_tx.clone(), db.clone()).unwrap();
    let wakeup = input::jobs::WakeUp::setup(db.clone(), joblist.clone()).unwrap();
    let (mpd_status, _mpd_watcher_thread, _updater_tx) =
        input::MpdStatus::start_updating().unwrap();

    let _controller_thread =
        controller::start(controller_rx, mpd_status.clone(), wakeup.clone()).unwrap();
    let _webserver = input::api::setup(wakeup.clone()).unwrap();

    loop {
        std::thread::park();
    }
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
