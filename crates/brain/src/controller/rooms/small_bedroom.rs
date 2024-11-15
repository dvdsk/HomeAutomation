use std::time::Duration;

use futures_concurrency::future::Race;
use futures_util::FutureExt;
use protocol::small_bedroom::ButtonPanel;
use protocol::{small_bedroom, Reading};
use tokio::sync::broadcast;
use tokio::time::{sleep_until, Instant};
use zigbee_bridge::lights::{denormalize, kelvin_to_mired};

use crate::controller::{local_now, Event, RestrictedSystem};

enum State {
    _Sleep,
    _Wakeup,
    Normal,
    _Away,
}

const INTERVAL: Duration = Duration::from_secs(5);

trait RecvFiltered {
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
            let event = self.recv().await.unwrap();
            if let Some(relevant) = filter_map(event) {
                return relevant;
            }
        }
    }
}

#[derive(Debug)]
enum RelevantEvent {
    Button(ButtonPanel),
}

fn event_filter(event: Event) -> Option<RelevantEvent> {
    match event {
        Event::Sensor(Reading::SmallBedroom(
            small_bedroom::Reading::ButtonPanel(button),
        )) => Some(RelevantEvent::Button(button)),
        _ => None,
    }
}

#[derive(Debug)]
enum Trigger {
    Event(RelevantEvent),
    ShouldUpdate,
}

pub async fn run(
    mut event_rx: broadcast::Receiver<Event>,
    // todo if state change message everyone using this
    _event_tx: broadcast::Sender<Event>,
    mut system: RestrictedSystem,
) {
    let _state = State::Normal;
    let mut next_update = Instant::now() + INTERVAL;
    loop {
        let get_event = event_rx
            .recv_filter_mapped(event_filter)
            .map(Trigger::Event);
        let tick = sleep_until(next_update).map(|_| Trigger::ShouldUpdate);

        let trigger = (get_event, tick).race().await;
        match trigger {
            Trigger::Event(RelevantEvent::Button(button)) => {
                handle_buttonpress(&mut system, button).await;
            }
            Trigger::ShouldUpdate => {
                update(&mut system).await;
                next_update = Instant::now() + INTERVAL;
            }
        }
    }
}

async fn handle_buttonpress(
    system: &mut RestrictedSystem,
    button: ButtonPanel,
) {
    dbg!(&button);
    match button {
        ButtonPanel::BottomLeft(_) => system.all_lamps_off().await,
        ButtonPanel::BottomMiddle(_) => system.all_lamps_on().await,
        ButtonPanel::BOttomRight(_) => system.all_lamps_ct(2000, 254).await,
        _ => (),
    }
}

async fn update(system: &mut RestrictedSystem) {
    let (new_ct, new_bri) = optimal_ct_bri();
    system.all_lamps_ct(new_ct, new_bri).await;
}

pub(crate) fn optimal_ct_bri() -> (u16, u8) {
    let now = local_now();
    let (temp, bri) = match now.hour() {
        9..17 => (3500, 1.0),
        17..20 => (3000, 1.0),
        20..22 => (2500, 0.85),
        22.. | 0..9 => (2000, 0.4),
    };
    (kelvin_to_mired(temp).try_into().unwrap(), denormalize(bri))
}
