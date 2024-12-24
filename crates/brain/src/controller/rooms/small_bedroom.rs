use std::time::Duration;

use futures_concurrency::future::Race;
use futures_util::FutureExt;
use jiff::civil::Time;
use protocol::small_bedroom::ButtonPanel;
use tokio::sync::broadcast;
use tokio::time::{sleep_until, Instant};
use tracing::trace;

use self::filter::{recv_filtered, RelevantEvent, Trigger};
use self::state::Room;
pub(crate) use self::state::State;
use crate::controller::{Event, RestrictedSystem};
use crate::input::jobs::Job;
use crate::time;

mod filter;
mod state;

const UPDATE_INTERVAL: Duration = Duration::from_secs(5);
const OFF_DELAY: Duration = Duration::from_secs(60);
const WAKEUP_EXPIRATION: Duration = Duration::from_secs(60);
const NAP_TIME: Duration = Duration::from_secs(30 * 60);

const fn time(hour: i8, minute: i8) -> f64 {
    hour as f64 + minute as f64 / 60.
}
const T0_00: f64 = time(0, 0);
const T8_00: f64 = time(8, 0);
const T9_00: f64 = time(9, 0);
const T16_00: f64 = time(16, 0);
const T17_00: f64 = time(17, 0);
const T18_00: f64 = time(18, 0);
const T20_30: f64 = time(20, 30);
const T21_30: f64 = time(21, 30);
const T22_00: f64 = time(22, 0);
const T23_00: f64 = time(22, 0);

pub async fn run(
    mut event_rx: broadcast::Receiver<Event>,
    event_tx: broadcast::Sender<Event>,
    system: RestrictedSystem,
) {
    let mut room = Room::new(event_tx, system.clone());
    let mut next_update = Instant::now() + UPDATE_INTERVAL;

    let wakeup_job =
        Job::every_day_at(8, 47, Event::WakeupSB, Some(WAKEUP_EXPIRATION));

    let res = system
        .system
        .jobs
        .remove_all_with_event(Event::WakeupSB)
        .await;
    trace!("Removing old SB wakeup jobs returned: {res:#?}");
    let res = system.system.jobs.add(wakeup_job.clone()).await;
    trace!("Tried to add job for SB wakeup: {wakeup_job:#?}");
    trace!("Jobs returned: {res:#?}");

    loop {
        let get_event = recv_filtered(&mut event_rx);
        let tick = sleep_until(next_update).map(|_| Trigger::ShouldUpdate);

        let trigger = (get_event, tick).race().await;
        match trigger {
            Trigger::Event(RelevantEvent::Button(button)) => {
                handle_buttonpress(&mut room, button).await;
            }
            Trigger::Event(RelevantEvent::Wakeup) => room.to_wakeup().await,
            Trigger::ShouldUpdate => {
                room.all_lights_daylight().await;
                next_update = Instant::now() + UPDATE_INTERVAL;
            }
        }
    }
}

async fn handle_buttonpress(room: &mut Room, button: ButtonPanel) {
    match button {
        ButtonPanel::BottomLeft(_) => {
            room.to_sleep().await;
        }
        ButtonPanel::BottomMiddle(_) => {
            use crate::time;
            let now = time::now();
            match time(now.hour(), now.minute()) {
                T23_00.. | T0_00..T9_00 => room.to_nightlight().await,
                _ => room.to_daylight().await,
            }
        }
        ButtonPanel::BOttomRight(_) => {
            room.to_override().await;
        }
        _ => (),
    }
}

pub(super) fn is_nap_time() -> bool {
    let now = time::now().datetime().time();

    now > Time::new(13, 0, 0, 0).unwrap()
        && now < Time::new(20, 0, 0, 0).unwrap()
}

// TODO: move to jobs system and remove update trigger
pub(super) fn daylight_now() -> (usize, f64) {
    let now = crate::time::now();

    match time(now.hour(), now.minute()) {
        T8_00..T9_00 => (2000, 0.5),
        T9_00..T16_00 => (4000, 1.0),
        T16_00..T17_00 => (3500, 1.0),
        T17_00..T18_00 => (2800, 1.0),
        T18_00..T20_30 => (2300, 1.0),
        T20_30..T21_30 => (2000, 0.7),
        T21_30..T22_00 => (1900, 0.4),
        T22_00.. | T0_00..T8_00 => (1800, 0.1),
        _ => (2300, 1.0),
    }
}
