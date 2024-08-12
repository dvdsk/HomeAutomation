use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use log_store::api::Percentile;
use protocol::Device;
use tokio::sync::Notify;
use tokio::time::sleep;
use tracing::{debug, warn};

use crate::tui::history_len::{self, HistoryLen};

use super::Fetching;

type Percentiles = Arc<Mutex<Vec<Percentile>>>;

#[derive(Debug)]
pub struct Stored {
    last_fetch: Option<Fetching>,
    last_update_started: Instant,
    pub data: Percentiles,
}

impl Stored {
    pub fn new() -> Self {
        Self {
            last_fetch: None,
            data: Arc::new(Mutex::new(Vec::new())),
            // will update immediately anyway since data len is zero
            last_update_started: Instant::now(),
        }
    }

    fn start_update(&mut self, device: Device, new_length: Duration, log_store: SocketAddr) {
        if let Some(in_progress) = self.last_fetch.take() {
            in_progress.cancel.notify_one();
            let _ = in_progress.handle.join();
            debug!("Canceled running stored-history update");
        }

        debug!("Started histogram fetch");
        let device = device.clone();
        let cancel = Arc::new(Notify::new());
        let cancelled = cancel.clone();
        let data = self.data.clone();
        let handle = thread::spawn(move || fetch(data, device, log_store, new_length, cancelled));
        self.last_fetch = Some(Fetching {
            handle,
            cancel,
            history_length: new_length,
        });
        self.last_update_started = Instant::now();
    }

    pub fn update_if_needed(
        &mut self,
        log_store: SocketAddr,
        device: Device,
        needed_hist: &mut HistoryLen,
    ) {
        if let Some(last_fetch) = &mut self.last_fetch {
            if last_fetch.history_length != needed_hist.dur || self.outdated() {
                self.start_update(device, needed_hist.dur, log_store);
                needed_hist.state = history_len::State::Fetching(Instant::now());
            }
        } else {
            self.start_update(device, needed_hist.dur, log_store);
            needed_hist.state = history_len::State::Fetching(Instant::now());
        }

        if let Some(last_fetch) = &mut self.last_fetch {
            if last_fetch.handle.is_finished() {
                needed_hist.state = history_len::State::Fetched
            }
        }
    }

    pub(crate) fn outdated(&self) -> bool {
        self.data.lock().unwrap().is_empty()
            || self.last_update_started.elapsed() > Duration::from_secs(5)
    }

    pub(crate) fn very_outdated(&self) -> bool {
        self.data.lock().unwrap().is_empty()
            || self.last_update_started.elapsed() > Duration::from_secs(15)
    }
}

fn fetch(
    histogram: Percentiles,
    device: Device,
    addr: SocketAddr,
    length: Duration,
    cancelled: Arc<Notify>,
) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    let Some(data) = rt.block_on(async {
        tokio::select! {
            data = connect_and_get_percentiles(addr, length, device) => Some(data),
            _ = cancelled.notified() => None,
        }
    }) else {
        debug!("Canceled data fetch");
        return;
    };

    let mut histogram = histogram.lock().unwrap();
    *histogram = data
}

async fn connect_and_get_percentiles(
    addr: SocketAddr,
    _: Duration,
    device: Device,
) -> Vec<Percentile> {
    let mut retry_period = Duration::ZERO;
    loop {
        sleep(retry_period).await;
        retry_period = Duration::from_secs(5)
            .min(retry_period * 2)
            .min(Duration::from_millis(100));

        let host = gethostname::gethostname();
        let host = host.to_string_lossy();
        let name = format!("sensor-tui@{host}");
        let mut api = match log_store::api::Client::connect(addr, name).await {
            Ok(api) => api,
            Err(e) => {
                warn!("Could not connect to data_store (at {addr}): {e}");
                continue;
            }
        };

        match api.get_percentiles(device.clone()).await {
            Ok(res) => return res,
            Err(e) => {
                warn!("Error getting data from data-store, reconnecting: {e}");
                continue;
            }
        };
    }
}
