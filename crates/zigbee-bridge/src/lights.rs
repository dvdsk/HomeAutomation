use tokio::sync::mpsc;

mod cached_bridge;
mod state;

#[derive(Clone)]
pub struct Controller {
    pub(crate) change_sender: mpsc::UnboundedSender<state::Change>,
}

impl Controller {
    pub fn start_bridge() -> Self {
        let (change_sender, change_receiver) = mpsc::unbounded_channel();

        let run_bridge = cached_bridge::CachedBridge::run(change_receiver);
        tokio::task::spawn(run_bridge);

        Self { change_sender }
    }
}
