use std::time::Duration;

use futures_concurrency::future::Race;
use futures_util::FutureExt;
use jiff::civil::Time;
use tokio::sync::broadcast;
use tokio::time::{sleep_until, Instant};
use tracing::warn;

use crate::controller::rooms::small_bedroom;
use crate::controller::{Event, RestrictedSystem};
use crate::time;

#[derive(PartialEq, Eq)]
enum State {
    Sleep,
    Daylight,
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
    Sleep,
    Daylight,
}

fn filter(event: Event) -> Option<RelevantEvent> {
    match event {
        Event::StateChangeSB(small_bedroom::State::Sleep) => {
            Some(RelevantEvent::Sleep)
        }
        Event::StateChangeSB(small_bedroom::State::Daylight) => {
            Some(RelevantEvent::Daylight)
        }
        _ => None,
    }
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

    let mut state = State::Daylight;
    let mut next_update = Instant::now() + INTERVAL;
    loop {
        let get_event = event_rx.recv_filter_mapped(filter).map(Res::Event);
        let tick = sleep_until(next_update).map(|_| Res::ShouldUpdate);

        let res = (get_event, tick).race().await;
        match res {
            Res::Event(RelevantEvent::Sleep) => {
                state = State::Sleep;
                if small_bedroom::is_nap_time() {
                    system.all_lamps_off().await;
                } else {
                    system.all_lamps_but_one_off("kitchen:hallway").await;
                }
            }
            Res::Event(RelevantEvent::Daylight) => {
                state = State::Daylight;
                update(&mut system).await;
                // TODO: only when LB also awake
                // then turn all lamps off when SB sleep
                system.all_lamps_on().await;
            }
            Res::ShouldUpdate if state == State::Daylight => {
                update(&mut system).await;
                system.all_lamps_on().await;
                next_update = Instant::now() + INTERVAL;
            }
            _ => (),
        }
    }
}

async fn update(system: &mut RestrictedSystem) {
    let (new_ct, new_bri) = small_bedroom::daylight_now();
    // let (new_ct, new_bri) = _testing_ct_bri();
    system.all_lamps_ct(new_ct, new_bri).await;
    tracing::trace!("updated lamps");
}

fn _testing_ct_bri() -> (usize, f64) {
    let now = crate::time::now();
    // let optimal = match now.hour() {
    let optimal = match now.minute() {
        min if min % 2 == 0 => (2000, 1.0), // Even hour: orange
        min if min % 2 == 1 => (4000, 1.0), // Odd hour: blue
        _ => (2000, 1.0),
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
