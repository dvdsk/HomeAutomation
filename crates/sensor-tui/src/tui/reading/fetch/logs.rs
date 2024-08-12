use log_store::api::ErrorEvent;
use protocol::Reading;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use tokio::time::sleep;
use tracing::{debug, warn};

use crate::tui::history_len::{self, HistoryLen};

use super::Fetching;

type Data = Arc<Mutex<Vec<log_store::api::ErrorEvent>>>;

#[derive(Debug)]
pub struct Stored {
    last_fetch: Option<Fetching>,
    pub logs: Data,
}

impl Stored {
    pub fn new() -> Self {
        Self {
            last_fetch: None,
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn list(&self) -> Vec<log_store::api::ErrorEvent> {
        self.logs.lock().unwrap().clone()
    }

    fn start_update(&mut self, reading: Reading, new_length: Duration, log_store: SocketAddr) {
        if let Some(in_progress) = self.last_fetch.take() {
            in_progress.cancel.notify_one();
            let _ = in_progress.handle.join();
            debug!("Canceled running stored-history update");
        }

        let reading = reading.clone();
        let cancel = Arc::new(Notify::new());
        let cancelled = cancel.clone();
        let data = self.logs.clone();
        let handle = thread::spawn(move || fetch(data, reading, log_store, new_length, cancelled));
        self.last_fetch = Some(Fetching {
            handle,
            cancel,
            history_length: new_length,
        });
    }

    pub fn update_if_needed(
        &mut self,
        log_store: SocketAddr,
        reading: Reading,
        needed_hist: &mut HistoryLen,
    ) {
        if let Some(last_fetch) = &mut self.last_fetch {
            if last_fetch.history_length != needed_hist.dur {
                self.start_update(reading, needed_hist.dur, log_store);
                needed_hist.state = history_len::State::Fetching(Instant::now());
            }
        } else {
            self.start_update(reading, needed_hist.dur, log_store);
            needed_hist.state = history_len::State::Fetching(Instant::now());
        }

        if let Some(last_fetch) = &self.last_fetch {
            if last_fetch.handle.is_finished() {
                needed_hist.state = history_len::State::Fetched
            }
        }
    }
}

fn fetch(
    logs: Arc<Mutex<Vec<ErrorEvent>>>,
    reading: Reading,
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
            data = connect_and_get_data(addr, length, reading) => Some(data),
            _ = cancelled.notified() => None,
        }
    }) else {
        debug!("Canceled data fetch");
        return;
    };

    let mut logs = logs.lock().unwrap();
    *logs = data;
}

async fn connect_and_get_data(
    addr: SocketAddr,
    _length: Duration,
    reading: Reading,
) -> Vec<ErrorEvent> {
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

        match api.get_logs(reading.device()).await {
            Ok(res) => return res,
            Err(e) => {
                warn!("Error getting data from data-store, reconnecting: {e}");
                continue;
            }
        };
    }
}
