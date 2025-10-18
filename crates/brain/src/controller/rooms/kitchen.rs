use std::time::Duration;

use futures_concurrency::future::Race;
use futures_util::FutureExt;
use tokio::sync::broadcast;
use tokio::time::{sleep_until, Instant};
use tracing::{trace, warn};

use crate::controller::rooms::common::RecvFiltered;
use crate::controller::rooms::small_bedroom;
use crate::controller::{Event, RestrictedSystem};
use crate::input::jobs::Job;

const INTERVAL: Duration = Duration::from_secs(5);

#[derive(PartialEq, Eq)]
enum State {
    Sleep,
    Daylight,
    Override,
    Wakeup,
}

#[derive(Debug)]
enum RelevantEvent {
    Sleep,
    Daylight,
    Override,
    Wakeup,
}

fn filter(event: Event) -> Option<RelevantEvent> {
    match event {
        Event::StateChangeSB(small_bedroom::State::Sleep) => {
            Some(RelevantEvent::Sleep)
        }
        Event::StateChangeSB(small_bedroom::State::Daylight) => {
            Some(RelevantEvent::Daylight)
        }
        Event::StateChangeSB(small_bedroom::State::Override) => {
            Some(RelevantEvent::Override)
        }
        Event::WakeupKitchen => Some(RelevantEvent::Wakeup),
        _ => None,
    }
}

pub async fn run(
    mut event_rx: broadcast::Receiver<Event>,
    // todo if state change message everyone using this
    _event_tx: broadcast::Sender<Event>,
    mut system: RestrictedSystem,
) {
    #[derive(Debug)]
    enum Res {
        Event(RelevantEvent),
        ShouldUpdate,
    }

    let mut state = State::Daylight;
    let mut next_update = Instant::now() + INTERVAL;

    let wakeup_job = Job::every_day_at(10, 0, Event::WakeupKitchen, None);
    let res = system
        .system
        .jobs
        .remove_all_with_event(Event::WakeupKitchen)
        .await;
    trace!("Kitchen wakeup job remove result: {res:?}");
    let res = system.system.jobs.add(wakeup_job.clone()).await;
    trace!("Kitchen wakeup job add result: {res:?}");

    loop {
        let get_event = event_rx.recv_filter_mapped(filter).map(Res::Event);
        let tick = sleep_until(next_update).map(|_| Res::ShouldUpdate);

        let res = (get_event, tick).race().await;
        match res {
            Res::Event(RelevantEvent::Sleep) => {
                state = State::Sleep;
                system.all_lamps_off().await;
            }
            Res::Event(RelevantEvent::Wakeup) => {
                state = State::Wakeup;
                trace!("Starting kitchen wakeup");
                wakeup_some_lamps(&mut system).await;
            }
            Res::Event(RelevantEvent::Daylight) => {
                state = State::Daylight;
                update(&mut system).await;
                system.all_lamps_on().await;
            }
            Res::Event(RelevantEvent::Override) => {
                state = State::Override;
                system.all_lamps_ct(2000, 1.0).await;
                system.all_lamps_on().await;
            }
            Res::ShouldUpdate => {
                if state == State::Daylight {
                    update(&mut system).await;
                    system.all_lamps_on().await;
                }
                next_update = Instant::now() + INTERVAL;
            }
        }
    }
}

async fn wakeup_some_lamps(system: &mut RestrictedSystem) {
    const BRI: f64 = 10. / 254.;
    const CT: usize = 2000;

    system.one_lamp_ct("kitchen:hood_left", CT, BRI).await;
    system.one_lamp_ct("kitchen:hood_right", CT, BRI).await;
    system.one_lamp_ct("kitchen:fridge", CT, BRI).await;

    system.one_lamp_on("kitchen:hood_left").await;
    system.one_lamp_on("kitchen:hood_right").await;
    system.one_lamp_on("kitchen:fridge").await;
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
