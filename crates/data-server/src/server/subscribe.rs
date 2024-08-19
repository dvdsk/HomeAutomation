use std::time::Duration;

use color_eyre::eyre::eyre;
use futures::SinkExt;
use std::mem;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::debug;

use color_eyre::Result;

use super::{Conn, Event};
use crate::api::{Response, SubMessage};

pub async fn handle_subscriber(
    sub: &mut Conn,
    mut rx: mpsc::Receiver<SubMessage>,
) -> color_eyre::Report {
    loop {
        let update = rx.recv().await.expect("updates always keep coming");
        let message = Response::SubUpdate(update);
        if timeout(Duration::from_secs(2), sub.send(message))
            .await
            .is_err()
        {
            return eyre!("Timed out while sending update");
        }
    }
}

pub async fn spread_updates(mut events: mpsc::Receiver<Event>) -> Result<()> {
    let mut subscribers = Vec::new();

    loop {
        let event = events
            .recv()
            .await
            .expect("queue is kept open by register_subs");
        debug!("event: {event:?}");

        let msg = match event {
            Event::NewSub { tx } => {
                subscribers.push(tx);
                continue;
            }
            Event::NewReading(Ok(reading)) => SubMessage::Reading(reading),
            Event::NewReading(Err(err)) => SubMessage::ErrorReport(err),
        };

        let subs = mem::take(&mut subscribers);
        for sub in subs {
            if sub.try_send(msg.clone()).is_ok() {
                subscribers.push(sub);
            }
        }
    }
}
