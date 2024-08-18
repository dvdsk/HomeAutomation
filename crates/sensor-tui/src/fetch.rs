use std::collections::VecDeque;
use std::future::Future;
use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::sync::mpsc;
use std::time::Instant;

use color_eyre::Result;
use jiff::{Span, Timestamp};
use log_store::api::{ErrorEvent, Percentile};
use protocol::Reading;
use tokio::task;
use tracing::debug;

use crate::tui::history_len;
use crate::{client_name, Fetchable, Update};

const MAX_IN_FLIGHT_REQUESTS: usize = 6;

#[derive(Debug, Clone)]
pub struct Data {
    reading: Reading,
    range: RangeInclusive<Timestamp>,
}

#[derive(Debug, Clone)]
pub struct Logs {
    reading: Reading,
    range: RangeInclusive<Timestamp>,
}

#[derive(Debug, Clone)]
pub struct Hist {
    reading: Reading,
    range: RangeInclusive<Timestamp>,
}

#[derive(Debug, Clone)]
pub enum Request {
    Data(Data),
    Logs(Logs),
    Hist(Hist),
}

impl Request {
    fn unwrap_logs(&self) -> Option<&Logs> {
        match self {
            Request::Logs(d) => Some(d),
            _ => None,
        }
    }
    fn unwrap_data(&self) -> Option<&Data> {
        match self {
            Request::Data(d) => Some(d),
            _ => None,
        }
    }
    fn unwrap_hist(&self) -> Option<&Hist> {
        match self {
            Request::Hist(d) => Some(d),
            _ => None,
        }
    }
}

pub struct Fetch {
    pub recently_issued: VecDeque<Request>,
    pub tx: tokio::sync::mpsc::Sender<Request>,
}

impl Fetch {
    pub fn new(
        data_store: SocketAddr,
        log_store: SocketAddr,
        update_tx: mpsc::Sender<Update>,
    ) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        task::spawn(handle_requests(data_store, log_store, rx, update_tx));
        Self {
            recently_issued: VecDeque::new(),
            tx,
        }
    }

    pub fn assure_up_to_date(
        &mut self,
        history_len: &mut history_len::HistoryLen,
        reading: Reading,
        oldest_in_data: Timestamp,
        logs_cover_from: Option<Timestamp>,
        hist_cover: Option<RangeInclusive<Timestamp>>,
    ) {
        let history_spans = Span::try_from(history_len.dur).unwrap();
        let start_needed = Timestamp::now() - history_spans;
        if self.history_outdated_not_updating(&reading, start_needed, oldest_in_data) {
            debug!("Requesting history for {reading:?}");
            self.request(Request::Data(Data {
                reading: reading.clone(),
                range: start_needed..=Timestamp::now(),
            }));
            history_len.state = history_len::State::Fetching(Instant::now());
        }
        if self.logs_outdated_not_updating(&reading, start_needed, logs_cover_from) {
            debug!("Requesting logs for {reading:?}");
            self.request(Request::Logs(Logs {
                reading: reading.clone(),
                range: start_needed..=Timestamp::now(),
            }))
        }
        if self.hist_outdated_not_updating(&reading, start_needed, hist_cover) {
            debug!("Requesting percentiles for {reading:?}");
            self.request(Request::Hist(Hist {
                reading: reading.clone(),
                range: start_needed..=Timestamp::now(),
            }))
        }
    }

    fn history_outdated_not_updating(
        &mut self,
        reading: &Reading,
        oldest_needed: Timestamp,
        oldest_in_history: Timestamp,
    ) -> bool {
        if oldest_in_history <= oldest_needed {
            return false;
        }

        let with_margin = oldest_needed.as_millisecond() as f64 * 1.2;
        let not_too_old = |start: &i64| *start < with_margin as i64;

        !self
            .recently_issued
            .iter()
            .filter_map(Request::unwrap_data)
            .filter(|req| &req.reading == reading)
            .map(|req| req.range.start().as_millisecond())
            .filter(not_too_old)
            .any(|start| start < oldest_needed.as_millisecond())
    }

    fn logs_outdated_not_updating(
        &mut self,
        reading: &Reading,
        oldest_needed: Timestamp,
        logs_cover_from: Option<Timestamp>,
    ) -> bool {
        if logs_cover_from.is_some_and(|oldest| oldest <= oldest_needed) {
            return false;
        }

        !self
            .recently_issued
            .iter()
            .filter_map(Request::unwrap_logs)
            .filter(|req| &req.reading == reading)
            .map(|req| req.range.start().as_millisecond())
            .any(|start| start < oldest_needed.as_millisecond())
    }

    fn hist_outdated_not_updating(
        &mut self,
        reading: &Reading,
        oldest_needed: Timestamp,
        hist_range: Option<RangeInclusive<Timestamp>>,
    ) -> bool {
        fn covers_recently(end: Timestamp) -> bool {
            Timestamp::now().since(end).unwrap().get_seconds() < 5
        }

        if hist_range.is_some_and(|r| r.contains(&oldest_needed) && covers_recently(*r.end())) {
            return false;
        }

        !self
            .recently_issued
            .iter()
            .filter_map(Request::unwrap_hist)
            .filter(|req| &req.reading == reading)
            .map(|req| req.range.clone())
            .filter(|range| covers_recently(*range.end()))
            .map(|range| range.start().as_millisecond())
            .any(|start| start < oldest_needed.as_millisecond())
    }

    fn request(&mut self, req: Request) {
        self.tx.blocking_send(req.clone()).unwrap();
        self.recently_issued.push_front(req);
        // might get out of sync with request handler, that is okay
        // false positives in self.recently_issued are allowed. They only
        // delay an update by a single frame.
        if self.recently_issued.len() > MAX_IN_FLIGHT_REQUESTS {
            self.recently_issued.pop_back();
        }
    }
}

pub(crate) async fn handle_requests(
    data_store: SocketAddr,
    log_store: SocketAddr,
    mut rx: tokio::sync::mpsc::Receiver<Request>,
    tx: mpsc::Sender<Update>,
) {
    let mut inflight_request = VecDeque::new();
    while let Some(request) = rx.recv().await {
        let data_store = data_store.clone();
        let tx = tx.clone();
        let handle = match request {
            Request::Data(Data { reading, range }) => tokio::spawn(get_wrap_send(
                get_data(data_store, reading.clone(), range),
                |res| match res {
                    Ok(data) => Update::Fetched {
                        reading,
                        thing: Fetchable::Data {
                            timestamps: data.0,
                            data: data.1,
                        },
                    },
                    Err(err) => Update::FetchError(err),
                },
                tx,
            )),
            Request::Logs(Logs { reading, range }) => tokio::spawn(get_wrap_send(
                get_logs(log_store, reading.clone()),
                move |res| match res {
                    Ok(logs) => Update::Fetched {
                        reading,
                        thing: Fetchable::Logs {
                            logs,
                            start_at: *range.start(),
                        },
                    },
                    Err(err) => Update::FetchError(err),
                },
                tx,
            )),
            Request::Hist(Hist { reading, range }) => tokio::spawn(get_wrap_send(
                get_percentiles(log_store, reading.clone()),
                move |res| match res {
                    Ok(percentiles) => Update::Fetched {
                        reading,
                        thing: Fetchable::Hist { percentiles, range },
                    },
                    Err(err) => Update::FetchError(err),
                },
                tx,
            )),
        };
        inflight_request.push_front(handle);
        if inflight_request.len() > 6 {
            let to_cancel = inflight_request.pop_back().expect("more then 0 items");
            to_cancel.abort();
        }
    }
}

pub async fn get_wrap_send<T>(
    getter: impl Future<Output = T>,
    wrapper: impl FnOnce(T) -> Update,
    tx: mpsc::Sender<Update>,
) {
    let val = getter.await;
    let update = (wrapper)(val);
    tx.send(update).unwrap();
}

pub async fn get_data(
    data_store: SocketAddr,
    reading: Reading,
    range: RangeInclusive<Timestamp>,
) -> Result<(Vec<Timestamp>, Vec<f32>)> {
    let mut api = data_store::api::Client::connect(data_store, client_name()).await?;

    let history = api
        .get_data(*range.start(), *range.end(), reading, 300)
        .await?;
    Ok(history)
}

pub async fn get_logs(log_store: SocketAddr, reading: Reading) -> Result<Vec<ErrorEvent>> {
    let mut api = log_store::api::Client::connect(log_store, client_name()).await?;

    let history = api.get_logs(reading.device()).await?;
    Ok(history)
}

pub async fn get_percentiles(log_store: SocketAddr, reading: Reading) -> Result<Vec<Percentile>> {
    let mut api = log_store::api::Client::connect(log_store, client_name()).await?;

    let percentile = api.get_percentiles(reading.device()).await?;
    Ok(percentile)
}
