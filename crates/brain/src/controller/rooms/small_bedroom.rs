use std::time::Duration;

use futures_concurrency::future::Race;
use futures_util::FutureExt;
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
    // DeskButton(protocol::large_bedroom::DeskButton),
}

fn filter(_event: Event) -> Option<RelevantEvent> {
    // match event {
    //     // Event::Sensor(SensorValue::ButtonPress(Button::LargeBedroomDesk(desk))) => {
    //     //     Some(RelevantEvent::DeskButton(desk))
    //     // }
    //     _ => None,
    // }
    None
}

pub async fn run(
    mut event_rx: broadcast::Receiver<Event>,
    // todo if state change message everyone using this
    _event_tx: broadcast::Sender<Event>,
    mut system: RestrictedSystem,
) {
    enum Res {
        Event(RelevantEvent),
        ShouldUpdate,
    }

    let _state = State::Normal;
    let mut next_update = Instant::now() + INTERVAL;
    loop {
        let get_event = event_rx.recv_filter_mapped(filter).map(Res::Event);
        let tick = sleep_until(next_update).map(|_| Res::ShouldUpdate);

        let res = (get_event, tick).race().await;
        match res {
            Res::Event(_) => (), //handle_event(e),
            Res::ShouldUpdate => {
                update(&mut system).await;
                next_update = Instant::now() + INTERVAL;
            }
        }
    }
}

async fn update(system: &mut RestrictedSystem) {
    let (new_ct, new_bri) = optimal_ct_bri();
    system.all_lamps_ct(new_ct, new_bri).await;
}

const fn time(hour: u8, minute: u8) -> f64 {
    hour as f64 + minute as f64 / 60.
}

pub(crate) fn optimal_ct_bri() -> (u16, u8) {
    let now = local_now();
    const T0_00: f64 = time(0, 0);
    const T8_00: f64 = time(8, 0);
    const T9_00: f64 = time(9, 0);
    const T17_00: f64 = time(17, 0);
    const T20_30: f64 = time(20, 30);
    const T21_30: f64 = time(21, 30);
    const T22_00: f64 = time(22, 0);

    let (temp, bri) = match time(now.hour(), now.minute()) {
        T8_00..T9_00 => (1500, 0.5),
        T9_00..T17_00 => (3500, 1.0),
        T17_00..T20_30 => (2500, 1.0),
        T20_30..T21_30 => (2000, 0.8),
        T21_30..T22_00 => (1500, 0.5),
        T22_00.. | T0_00..T8_00 => (1000, 0.1),
        _ => (2500, 1.0),
    };
    (kelvin_to_mired(temp).try_into().unwrap(), denormalize(bri))
}
