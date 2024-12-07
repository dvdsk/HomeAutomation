use std::time::Duration;

use futures_concurrency::future::Race;
use futures_util::FutureExt;
use protocol::small_bedroom::ButtonPanel;
use tokio::sync::broadcast;
use tokio::time::{sleep_until, Instant};
use tracing::warn;

use self::filter::{recv_filtered, RelevantEvent, Trigger};
use self::state::Room;
pub(crate) use self::state::State;
use crate::controller::{Event, RestrictedSystem};
use crate::input::jobs::Job;

mod filter;
mod state;

const UPDATE_INTERVAL: Duration = Duration::from_secs(5);
const OFF_DELAY: Duration = Duration::from_secs(60);
const WAKEUP_EXPIRATION: Duration = Duration::from_secs(60 * 60);

pub async fn run(
    mut event_rx: broadcast::Receiver<Event>,
    event_tx: broadcast::Sender<Event>,
    system: RestrictedSystem,
) {
    let mut room = Room::new(event_tx, system.clone());
    let mut next_update = Instant::now() + UPDATE_INTERVAL;

    let _ = system
        .system
        .jobs
        .add(Job::every_day_at(
            9,
            0,
            Event::WakeupSB,
            Some(WAKEUP_EXPIRATION),
        ))
        .await;
    warn!("Added job for SB wakeup");

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
    dbg!(button);
    match button {
        ButtonPanel::BottomLeft(_) => {
            room.to_sleep().await;
        }
        ButtonPanel::BottomMiddle(_) => {
            room.to_daylight().await;
        }
        ButtonPanel::BOttomRight(_) => {
            room.to_override().await;
        }
        _ => (),
    }
}

const fn time(hour: i8, minute: i8) -> f64 {
    hour as f64 + minute as f64 / 60.
}

// TODO: move to jobs system and remove update trigger
pub(super) fn daylight_now() -> (usize, f64) {
    let now = crate::time::now();

    const T0_00: f64 = time(0, 0);
    const T8_00: f64 = time(8, 0);
    const T9_00: f64 = time(9, 0);
    const T17_00: f64 = time(17, 0);
    const T20_30: f64 = time(20, 30);
    const T21_30: f64 = time(21, 30);
    const T22_00: f64 = time(22, 0);

    match time(now.hour(), now.minute()) {
        T8_00..T9_00 => (2000, 0.5),
        T9_00..T17_00 => (3500, 1.0),
        T17_00..T20_30 => (2300, 1.0),
        T20_30..T21_30 => (2000, 0.7),
        T21_30..T22_00 => (1900, 0.4),
        T22_00.. | T0_00..T8_00 => (1800, 0.1),
        _ => (2300, 1.0),
    }
}
