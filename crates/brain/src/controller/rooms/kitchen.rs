use std::time::Duration;

use futures_concurrency::future::Race;
use futures_util::FutureExt;
use tokio::sync::broadcast;
use tokio::time::{sleep_until, Instant};
use tracing::warn;

use crate::controller::rooms::small_bedroom;
use crate::controller::{Event, RestrictedSystem};

enum State {
    // Sleep,
    // Wakeup,
    Normal,
    // Away,
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
            Res::Event(_) => (), // handle_event(e),
            Res::ShouldUpdate => {
                update(&mut system).await;
                next_update = Instant::now() + INTERVAL;
            }
        }
    }
}

async fn update(system: &mut RestrictedSystem) {
    let (new_ct, new_bri) = small_bedroom::optimal_ct_bri();
    // let (new_ct, new_bri) = _testing_ct_bri();
    system.all_lamps_ct(new_ct, new_bri).await;
    tracing::trace!("updated lamps");
}

fn _testing_ct_bri() -> (u16, u8) {
    let now = crate::time::now();
    // let optimal = match now.hour() {
    let optimal = match now.minute() {
        min if min % 2 == 0 => (400, u8::MAX), // Even hour: orange
        min if min % 2 == 1 => (250, u8::MAX), // Odd hour: blue
        _ => (400, u8::MAX),
    };
    // if now.minute() == 0 && now.second() <= 9 {
    if now.second() <= 9 {
        warn!("B: correct color temp is now {}", optimal.0);
    }
    optimal
}

// fn handle_event(e: RelevantEvent) {
//     use protocol::large_bedroom::DeskButton as D;
//     use RelevantEvent as R;
//
//     match e {
//         R::DeskButton(D::OneOfFour(p)) if p.is_long() => todo!(),
//         unhandled => warn!("Unhandled button: {unhandled:?}"),
//     }
// }
