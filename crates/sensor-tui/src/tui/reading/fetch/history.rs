use protocol::Reading;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::Notify;
use tokio::time::sleep;
use tracing::{debug, warn};

use crate::client_name;
use crate::tui::history_len::{self, HistoryLen};

use super::Fetching;

type Data = Arc<Mutex<Vec<(jiff::Timestamp, f32)>>>;

#[derive(Debug)]
pub struct Stored {
    last_fetch: Option<Fetching>,
    pub data: Data,
}

impl Stored {
    pub fn new() -> Self {
        Self {
            last_fetch: None,
            data: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn start_update(&mut self, reading: Reading, new_length: Duration, data_store: SocketAddr) {
        if let Some(in_progress) = self.last_fetch.take() {
            in_progress.cancel.notify_one();
            let _ = in_progress.handle.join();
            debug!("Canceled running stored-history update");
        }

        let reading = reading.clone();
        let cancel = Arc::new(Notify::new());
        let cancelled = cancel.clone();
        let data = self.data.clone();
        let handle = thread::spawn(move || fetch(data, reading, data_store, new_length, cancelled));
        self.last_fetch = Some(Fetching {
            handle,
            cancel,
            history_length: new_length,
        });
    }

    pub fn update_if_needed(
        &mut self,
        data_store: SocketAddr,
        reading: Reading,
        needed_hist: &mut HistoryLen,
    ) {
        if let Some(last_fetch) = &mut self.last_fetch {
            if last_fetch.history_length != needed_hist.dur {
                self.start_update(reading, needed_hist.dur, data_store);
                needed_hist.state = history_len::State::Fetching(Instant::now());
            }
        } else {
            self.start_update(reading, needed_hist.dur, data_store);
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
    history: Arc<Mutex<Vec<(jiff::Timestamp, f32)>>>,
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

    debug!(
        "got data from {:?} till {:?}",
        data.0.first(),
        data.0.last()
    );
    let mut history = history.lock().unwrap();
    history.clear();
    for (t, y) in data.0.into_iter().zip(data.1) {
        history.push((t, y))
    }
    debug!("done fetching {length:?}!");
}

async fn connect_and_get_data(
    addr: SocketAddr,
    length: Duration,
    reading: Reading,
) -> (Vec<jiff::Timestamp>, Vec<f32>) {
    let mut retry_period = Duration::ZERO;
    loop {
        sleep(retry_period).await;
        retry_period = Duration::from_secs(5)
            .min(retry_period * 2)
            .max(Duration::from_millis(100));

        let mut api = match data_store::api::Client::connect(addr, client_name()).await {
            Ok(api) => api,
            Err(e) => {
                warn!("Could not connect to data_store (at {addr}): {e}");
                continue;
            }
        };

        match api
            .get_data(
                jiff::Timestamp::now()
                    - jiff::Span::default().milliseconds(length.as_millis() as i64),
                jiff::Timestamp::now(),
                reading.clone(),
                300,
            )
            .await
        {
            Ok(res) => return res,
            Err(e) => {
                warn!("Error getting data from data-store, reconnecting: {e}");
                continue;
            }
        };
    }
}
