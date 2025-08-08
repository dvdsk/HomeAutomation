use tokio::sync::broadcast::{self, error::RecvError};
use tracing::warn;

use crate::controller::Event;

pub(crate) trait RecvFiltered {
    async fn recv_filter_mapped<T>(
        &mut self,
        filter_map: impl Fn(Event) -> Option<T>,
    ) -> T;
}

impl RecvFiltered for broadcast::Receiver<Event> {
    async fn recv_filter_mapped<T>(
        &mut self,
        filter_map: impl Fn(Event) -> Option<T>,
    ) -> T {
        loop {
            let event = match self.recv().await {
                Ok(event) => event,
                Err(RecvError::Lagged(n)) => {
                    warn!("A room missed {n} events");
                    continue;
                }
                Err(err) => panic!("{err}"),
            };
            if let Some(relevant) = filter_map(event) {
                return relevant;
            }
        }
    }
}
