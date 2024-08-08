use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::Notify;

pub mod histogram;
pub mod history;

#[derive(Debug)]
struct Fetching {
    handle: thread::JoinHandle<()>,
    cancel: Arc<Notify>,
    history_length: Duration,
}
