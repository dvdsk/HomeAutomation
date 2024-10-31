#![allow(unused)]
use tokio::sync::mpsc;

mod cached_bridge;
mod conversion;
mod state;

#[derive(Clone)]
pub struct Controller {
    change_sender: mpsc::UnboundedSender<(String, state::Change)>,
}

impl Controller {
    pub async fn start_bridge() -> Self {
        let (change_sender, change_receiver) = mpsc::unbounded_channel();

        let run_bridge = cached_bridge::run(change_receiver);
        tokio::task::spawn(run_bridge);

        Self { change_sender }
    }
}
