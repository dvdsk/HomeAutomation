use std::collections::VecDeque;
use std::future::Future;
use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::sync::{mpsc, Arc};
use std::time::Duration;

use color_eyre::eyre::{self, Context, Report};
use color_eyre::Result;
use jiff::{Span, Timestamp, Unit};
use log_store::api::{ErrorEvent, Percentile};
use protocol::Reading;
use std::sync::Mutex;
use tokio::time::Instant;
use tracing::{debug, instrument};

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
    fn data(&self) -> Option<&Data> {
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

        tokio::spawn(handle_requests(data_store, log_store, rx, update_tx));
        Self {
            recently_issued: VecDeque::new(),
            tx,
        }
    }

    #[instrument(skip(self, on_fetch_start))]
    pub fn assure_up_to_date(
        &mut self,
        history_len: Duration,
        on_fetch_start: impl FnOnce(),
        reading: Reading,
        oldest_in_data: Timestamp,
        logs_cover_from: Timestamp,
        hist_cover: Option<RangeInclusive<Timestamp>>,
    ) {
        let history_spans = Span::try_from(history_len).unwrap();
        let start_needed = Timestamp::now() - history_spans;
        if self.history_outdated_not_updating(&reading, start_needed, oldest_in_data) {
            debug!("Requesting history for {reading:?}");
            self.request(Request::Data(Data {
                reading: reading.clone(),
                range: start_needed..=Timestamp::now(),
            }));
            (on_fetch_start)();
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

    #[instrument(skip(self))]
    fn history_outdated_not_updating(
        &mut self,
        reading: &Reading,
        oldest_needed: Timestamp,
        oldest_in_history: Timestamp,
    ) -> bool {
        let target_span = Timestamp::now() - oldest_needed;
        let target_span = target_span.total(Unit::Millisecond).expect("fits f64");
        let max_span = (target_span * 1.2) as i64;

        let oldest_allowed = Timestamp::now().as_millisecond() - max_span;
        let up_to_date = |start: &i64| *start >= oldest_allowed;

        let curr_up_to_date = up_to_date(&oldest_in_history.as_millisecond());
        let curr_range_correct = oldest_in_history <= oldest_needed;

        if curr_up_to_date && curr_range_correct {
            return false; // No need for update
        }

        // Is there an ongoing request that will fix this?
        !self
            .recently_issued
            .iter()
            .filter_map(Request::data)
            .filter(|req| req.reading.is_same_as(reading))
            .map(|req| req.range.start().as_millisecond())
            .filter(up_to_date)
            .any(|start| start <= oldest_needed.as_millisecond())
    }

    fn logs_outdated_not_updating(
        &mut self,
        reading: &Reading,
        oldest_needed: Timestamp,
        logs_cover_from: Timestamp,
    ) -> bool {
        if logs_cover_from <= oldest_needed {
            return false;
        }

        !self
            .recently_issued
            .iter()
            .filter_map(Request::unwrap_logs)
            .filter(|req| &req.reading == reading)
            .map(|req| req.range.start().as_millisecond())
            .any(|start| start <= oldest_needed.as_millisecond())
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
            .any(|start| start <= oldest_needed.as_millisecond())
    }

    fn request(&mut self, req: Request) {
        self.tx.blocking_send(req.clone()).unwrap();
        self.recently_issued.push_front(req);
        // Might get out of sync with request handler, that is okay
        // false positives in `self.recently_issued` are allowed. They only
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
    let data_store_queue = Queue::default();
    let log_store_queue = Queue::default();

    let mut inflight_request = VecDeque::new();
    while let Some(request) = rx.recv().await {
        let tx = tx.clone();
        let handle = match request {
            Request::Data(Data { reading, range }) => {
                let reading_clone = reading.clone();
                tokio::spawn(get_retry_then_wrap_send(
                    move || get_data(data_store, reading.clone(), range.clone()),
                    data_store_queue.clone(),
                    "Could not fetch data for graph",
                    move |res| match res {
                        Ok(data) => Update::Fetched {
                            reading: reading_clone,
                            thing: Fetchable::Data {
                                timestamps: data.0,
                                data: data.1,
                            },
                        },
                        Err(err) => Update::FetchError(err),
                    },
                    tx,
                ))
            }
            Request::Logs(Logs { reading, range }) => {
                let reading_clone = reading.clone();
                let range_clone = range.clone();
                tokio::spawn(get_retry_then_wrap_send(
                    move || get_logs(log_store, reading.clone(), range.clone()),
                    log_store_queue.clone(),
                    "Could not fetch logs",
                    move |res| match res {
                        Ok(logs) => Update::Fetched {
                            reading: reading_clone,
                            thing: Fetchable::Logs {
                                logs,
                                start_at: *range_clone.start(),
                            },
                        },
                        Err(err) => Update::FetchError(err),
                    },
                    tx,
                ))
            }
            Request::Hist(Hist { reading, range }) => {
                let reading_clone = reading.clone();
                tokio::spawn(get_retry_then_wrap_send(
                    move || get_percentiles(log_store, reading.clone()),
                    log_store_queue.clone(),
                    "Could not fetch percentiles for histogram",
                    move |res| match res {
                        Ok(percentiles) => Update::Fetched {
                            reading: reading_clone,
                            thing: Fetchable::Hist { percentiles, range },
                        },
                        Err(err) => Update::FetchError(err),
                    },
                    tx,
                ))
            }
        };
        inflight_request.push_front(handle);
        if inflight_request.len() > 6 {
            let to_cancel = inflight_request.pop_back().expect("more then 0 items");
            to_cancel.abort();
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Queue(Arc<Mutex<InnerLimits>>);

impl Queue {
    fn register(&self) -> usize {
        let mut this = self.0.lock().unwrap();
        let most_recent_request = &mut this.most_recent_request;
        *most_recent_request += 1;
        *most_recent_request
    }

    fn set_next_allowed(&self, at: Instant) {
        let mut this = self.0.lock().unwrap();
        this.next_allowed = Some(at);
    }

    fn our_turn(&self, id: usize) -> Result<Option<Instant>, Instant> {
        let this = self.0.lock().unwrap();
        if let Some(next_allowed) = this.next_allowed {
            let until = next_allowed.saturating_duration_since(Instant::now());

            if until.is_zero() {
                Ok(None)
            } else if this.most_recent_request == id {
                Ok(Some(next_allowed))
            } else {
                // Queue behind highest priority
                Err(next_allowed + Duration::from_millis(10))
            }
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Default)]
struct InnerLimits {
    next_allowed: Option<Instant>,
    most_recent_request: usize,
}

#[must_use]
enum GetResult<T> {
    Ok(T),
    Err(eyre::Error),
    RateLimited { allowed_in: Duration },
}

impl<T> From<Result<T, eyre::Report>> for GetResult<T> {
    fn from(value: Result<T, eyre::Report>) -> Self {
        match value {
            Ok(v) => Self::Ok(v),
            Err(e) => Self::Err(e),
        }
    }
}

async fn get_retry_then_wrap_send<T, F: Future<Output = GetResult<T>>>(
    getter: impl Fn() -> F,
    queue: Queue,
    err_text: &'static str,
    wrapper: impl FnOnce(Result<T, eyre::Report>) -> Update,
    tx: mpsc::Sender<Update>,
) {
    let id = queue.register();

    let update = loop {
        match queue.our_turn(id) {
            Ok(Some(allowed_at)) => {
                tokio::time::sleep_until(allowed_at).await;
            }
            Ok(None) => (),
            Err(recheck_at) => {
                tokio::time::sleep_until(recheck_at).await;
                continue;
            }
        }

        match getter().await {
            GetResult::Ok(val) => break wrapper(Ok(val)),
            GetResult::Err(e) => break wrapper(Err(e).wrap_err(err_text)),
            GetResult::RateLimited { allowed_in } => {
                queue.set_next_allowed(Instant::now() + allowed_in);
            }
        }
    };
    tx.send(update).unwrap();
}

async fn get_data(
    data_store: SocketAddr,
    reading: Reading,
    range: RangeInclusive<Timestamp>,
) -> GetResult<(Vec<Timestamp>, Vec<f32>)> {
    use data_store::api::{client::ConnectError, client::Error, Data, GetDataError};

    let mut api = match data_store::api::Client::connect(data_store, client_name()).await {
        Ok(api) => api,
        Err(ConnectError::RateLimited(d)) => return GetResult::RateLimited { allowed_in: d },
        Err(other) => {
            return GetResult::Err(Report::new(other).wrap_err("Could not connect to data-store"))
        }
    };

    match api
        .get_data(*range.start(), *range.end(), reading, 300)
        .await
    {
        Ok(Data { time, values }) => GetResult::Ok((time, values)),
        Err(Error::Request(GetDataError::NotFound))
        | Err(Error::Request(GetDataError::EmptyFile))
        | Err(Error::Request(GetDataError::StartAfterData))
        | Err(Error::Request(GetDataError::StopBeforeData))
        | Err(Error::Request(GetDataError::NotInStore { .. })) => {
            GetResult::Ok((Vec::new(), Vec::new()))
        }
        Err(other) => GetResult::Err(
            Report::new(other).wrap_err("Data-store returned an error to our request"),
        ),
    }
}

async fn get_logs(
    log_store: SocketAddr,
    reading: Reading,
    range: RangeInclusive<Timestamp>,
) -> GetResult<Vec<ErrorEvent>> {
    use log_store::api::client::{Client, ConnectError};

    let mut api = match Client::connect(log_store, client_name()).await {
        Ok(api) => api,
        Err(ConnectError::RateLimited(d)) => return GetResult::RateLimited { allowed_in: d },
        Err(other) => {
            return GetResult::Err(Report::new(other).wrap_err("Could not connect to log-store"))
        }
    };

    api.get_logs(reading.device(), range)
        .await
        .wrap_err("Log store returned an error to our request")
        .into()
}

async fn get_percentiles(log_store: SocketAddr, reading: Reading) -> GetResult<Vec<Percentile>> {
    use log_store::api::client::{Client, ConnectError};

    let mut api = match Client::connect(log_store, client_name()).await {
        Ok(api) => api,
        Err(ConnectError::RateLimited(d)) => return GetResult::RateLimited { allowed_in: d },
        Err(other) => {
            return GetResult::Err(Report::new(other).wrap_err("Could not connect to log-store"))
        }
    };

    api.get_percentiles(reading.device())
        .await
        .wrap_err("Log store returned an error to our request")
        .into()
}
