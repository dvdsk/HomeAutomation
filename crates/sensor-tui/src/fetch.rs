use std::collections::VecDeque;
use std::future::Future;
use std::net::SocketAddr;
use std::ops::RangeInclusive;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use color_eyre::Result;
use jiff::Timestamp;
use protocol::Reading;
use tracing::{debug, warn};

use crate::{client_name, Fetched, Update};

const MAX_IN_FLIGHT_REQUESTS: usize = 6;

#[derive(Debug, Clone)]
pub struct Data {
    reading: Reading,
    range: RangeInclusive<Timestamp>,
}

#[derive(Debug, Clone)]
pub enum Request {
    Data(Data),
}

impl Request {
    fn unwrap_data(&self) -> Option<&Data> {
        match self {
            Request::Data(d) => Some(d),
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
    ) -> (Self, thread::JoinHandle<()>) {
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        let this = Self {
            recently_issued: VecDeque::new(),
            tx,
        };

        let maintainer = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .enable_time()
                .build()
                .unwrap();
            warn!("yo");
            rt.block_on(handle_requests(data_store, log_store, rx, update_tx));
        });
        (this, maintainer)
    }

    pub fn assure_up_to_date(
        &mut self,
        reading: Reading,
        history_length: Duration,
        oldest_in_data: Timestamp,
    ) {
        let history_len = jiff::Span::try_from(history_length).unwrap();
        let start_needed = Timestamp::now() - history_len;
        if self.history_outdated_not_updating(&reading, start_needed, oldest_in_data) {
            self.request(Request::Data(Data {
                reading,
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

    fn request(&mut self, req: Request) {
        debug!("sending request: {req:?}");
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
    _log_store: SocketAddr,
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
                    Ok(data) => Update::Fetched(Fetched::Data {
                        reading,
                        timestamps: data.0,
                        data: data.1,
                    }),
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
    unreachable!()
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
