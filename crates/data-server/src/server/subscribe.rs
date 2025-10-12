use protocol::reading::tree::Tree;
use protocol::reading::ReadingId;
use protocol::{pir, Reading};
use std::collections::HashMap;
use std::mem;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TrySendError;
use tokio_util::time::FutureExt;
use tracing::trace;

use color_eyre::Result;

use super::Event;
use crate::api::subscriber::SubMessage;

pub async fn handle_updates(mut events: mpsc::Receiver<Event>) -> Result<()> {
    let mut pir_state_tracker = PirStateTracker::new();
    let mut subscribers = Vec::new();

    loop {
        let update = events
            .recv()
            .timeout_at(pir_state_tracker.call_again_at.into())
            .await
            .map(|recv_res| {
                recv_res.expect("queue is kept open by register_subs")
            });
        trace!("update: {update:?}");

        match update {
            Err(_timeout) => {
                for pir_went_dark in pir_state_tracker.dark_pirs() {
                    broadcast_reading(
                        &mut subscribers,
                        SubMessage::Reading(pir_went_dark),
                    );
                }
            }
            Ok(event) => {
                spread_updates(&mut pir_state_tracker, &mut subscribers, event)
            }
        }
    }
}

fn spread_updates(
    pir_state_tracker: &mut PirStateTracker,
    subscribers: &mut Vec<mpsc::Sender<SubMessage>>,
    event: Event,
) {
    let to_forward = match event {
        Event::NewSub { tx } => {
            subscribers.push(tx);
            return;
        }
        Event::NewReading(Ok(reading)) => {
            if let Some(reading) =
                pir_state_tracker.intercept_pir_status(reading)
            {
                SubMessage::Reading(reading)
            } else {
                return;
            }
        }
        Event::NewReading(Err(err)) => SubMessage::ErrorReport(err),
        Event::AffectorControlled {
            affector,
            controlled_by,
        } => SubMessage::AffectorControlled {
            affector,
            controlled_by,
        },
    };
    broadcast_reading(subscribers, to_forward);
}

fn broadcast_reading(
    subscribers: &mut Vec<mpsc::Sender<SubMessage>>,
    to_forward: SubMessage,
) {
    let subs = mem::take(subscribers);
    for sub in subs {
        let res = sub.try_send(to_forward.clone());
        if let Ok(()) | Err(TrySendError::Full(_)) = res {
            subscribers.push(sub)
        }
    }
}

/// Pirs should send Status::Active every 5 seconds to confirm they have not
/// gone down/defective. It makes no sense to store Status::Active every 5
/// seconds.
///
/// - drops a Status::Active if one was already send
/// - inserts a Status::Unknown if no Status::Active was received in the last
///   6s. Unless a Status::InActive was received in between
struct PirStateTracker {
    active: HashMap<ReadingId, (Instant, Reading)>,
    call_again_at: Instant,
}

const ACTIVE_PIR_HEARTBEAT: Duration = Duration::from_secs(5);

impl PirStateTracker {
    fn new() -> Self {
        Self {
            active: HashMap::new(),
            call_again_at: Instant::now(),
        }
    }
    fn dark_pirs<'a>(&'a mut self) -> impl Iterator<Item = Reading> + 'a {
        self.active
            .extract_if(|_, (last_active, _)| {
                last_active.elapsed() > ACTIVE_PIR_HEARTBEAT
            })
            .map(|(_, (_, mut reading))| {
                let pir_status = reading
                    .value_mut()
                    .downcast_mut::<pir::Status>()
                    .expect("we only insert pir::Status");
                *pir_status = pir::Status::Unknown;
                reading
            })
    }

    fn intercept_pir_status(
        &mut self,
        mut reading: Reading,
    ) -> Option<Reading> {
        match reading.value_mut().downcast_ref() {
            Some(pir::Status::OngoingActivity) => {
                if self
                    .active
                    .insert(reading.id(), (Instant::now(), reading.clone()))
                    .is_none()
                {
                    Some(reading)
                } else {
                    None
                }
            }
            Some(pir::Status::NoActivity) => {
                self.active.remove(&reading.id());
                Some(reading)
            }
            Some(pir::Status::Unknown) => {
                unreachable!("Should never be send by node")
            }
            None => Some(reading),
        }
    }
}
