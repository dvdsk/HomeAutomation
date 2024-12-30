use tokio::sync::broadcast::{error::RecvError, Receiver};
use tracing::warn;

use crate::controller::Event;
use protocol::{small_bedroom::{self, portable_button_panel}, Reading};

#[derive(Debug)]
pub(super) enum Trigger {
    Event(RelevantEvent),
    ShouldUpdate,
}

#[derive(Debug)]
pub(super) enum RelevantEvent {
    Button(small_bedroom::ButtonPanel),
    PortableButton(portable_button_panel::Reading),
    Wakeup,
}

pub(super) async fn recv_filtered(event_rx: &mut Receiver<Event>) -> Trigger {
    loop {
        let event = match event_rx.recv().await {
            Ok(event) => event,
            Err(RecvError::Lagged(n)) => {
                warn!("SB missed {n} events");
                continue;
            }
            Err(err) => panic!("{err}"),
        };
        if let Some(relevant) = event_filter(event) {
            warn!("SB received relevant event: {relevant:?}");
            return Trigger::Event(relevant);
        }
    }
}

fn event_filter(event: Event) -> Option<RelevantEvent> {
    match event {
        Event::Sensor(Reading::SmallBedroom(
            small_bedroom::Reading::ButtonPanel(button),
        )) => Some(RelevantEvent::Button(button)),
        Event::WakeupSB => Some(RelevantEvent::Wakeup),
        _ => None,
    }
}
