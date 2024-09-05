use std::mem;
use tokio::sync::mpsc;
use tracing::trace;

use color_eyre::Result;

use super::Event;
use crate::api::subscriber::SubMessage;

pub async fn spread_updates(mut events: mpsc::Receiver<Event>) -> Result<()> {
    let mut subscribers = Vec::new();

    loop {
        let event = events
            .recv()
            .await
            .expect("queue is kept open by register_subs");
        trace!("event: {event:?}");

        let msg = match event {
            Event::NewSub { tx } => {
                subscribers.push(tx);
                continue;
            }
            Event::NewReading(Ok(reading)) => SubMessage::Reading(reading),
            Event::NewReading(Err(err)) => SubMessage::ErrorReport(err),
            Event::AffectorControlled {
                affector,
                controlled_by,
            } => SubMessage::AffectorControlled {
                affector,
                controlled_by,
            },
        };

        let subs = mem::take(&mut subscribers);
        for sub in subs {
            if sub.try_send(msg.clone()).is_ok() {
                subscribers.push(sub);
            }
        }
    }
}
