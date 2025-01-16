use std::collections::BTreeMap;
use std::fmt::Debug;
use std::time::Duration;

use futures_concurrency::future::Race;
use futures_util::FutureExt;
use jiff::civil::Time;
use protocol::small_bedroom::{portable_button_panel, ButtonPanel};
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
const RADIATOR_OVERRIDE_MINUTES: i32 = 60;

const fn time(hour: i8, minute: i8) -> f64 {
    hour as f64 + minute as f64 / 60.
}
const T0_00: f64 = time(0, 0);
const T9_00: f64 = time(9, 0);
const T23_00: f64 = time(23, 0);

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
            Trigger::Event(
                event @ RelevantEvent::Button(_)
                | event @ RelevantEvent::PortableButton(_),
            ) => {
                handle_buttonpress(&mut room, event).await;
            }
            Trigger::Event(RelevantEvent::RadiatorOverride) => {
                trace!("Starting radiator override");
                room.start_radiator_override();
            }
            Trigger::Event(RelevantEvent::Wakeup) => room.to_wakeup().await,
            Trigger::ShouldUpdate => {
                room.update_radiator().await;
                room.all_lights_daylight().await;
                next_update = Instant::now() + UPDATE_INTERVAL;
            }
        }
    }
}

async fn handle_buttonpress(room: &mut Room, event: RelevantEvent) {
    use portable_button_panel::Reading as P;
    use ButtonPanel as B;
    use RelevantEvent as E;

    match event {
        E::Button(B::BottomLeft(_)) => {
            room.to_sleep_delayed().await;
        }
        E::PortableButton(P::Dots1InitialPress) => {
            room.to_sleep_immediate().await
        }
        E::Button(B::BottomMiddle(_))
        | E::PortableButton(P::Dots2InitialPress) => {
            use crate::time;
            let now = time::now();
            match time(now.hour(), now.minute()) {
                T23_00.. | T0_00..T9_00 => room.to_nightlight().await,
                _ => room.to_daylight().await,
            }
        }
        E::Button(B::BOttomRight(_)) => {
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

pub(crate) fn goal_temp_now() -> f64 {
    let goals = BTreeMap::from([
        ((00, 00), 18.0),
        ((08, 30), 19.0),
        ((10, 00), 19.5),
        ((11, 00), 20.0),
        ((12, 00), 20.5),
        ((13, 00), 21.0),
        ((20, 00), 20.0),
        ((22, 00), 18.0),
    ]);

    goal_now(goals, 18.0)
}

// TODO: move to jobs system and remove update trigger
pub(super) fn daylight_now() -> (usize, f64) {
    let goals = BTreeMap::from([
        ((08, 00), (2000, 0.5)),
        ((09, 00), (4000, 1.0)),
        ((19, 00), (3800, 1.0)),
        ((19, 30), (3600, 1.0)),
        ((19, 45), (3300, 1.0)),
        ((20, 00), (3000, 1.0)),
        ((20, 15), (2500, 1.0)),
        ((20, 30), (2000, 1.0)),
        ((21, 30), (1900, 0.8)),
        ((21, 00), (1800, 0.5)),
    ]);

    goal_now(goals, (2300, 1.0))
}

fn goal_now<T: Debug + Clone>(goals: BTreeMap<(i8, i8), T>, default: T) -> T {
    let Some(first_goal) = goals.first_key_value() else {
        return default;
    };
    let mut prev_goal_val = first_goal.1.clone();

    for ((h, m), goal_val) in goals {
        let now = crate::time::now().datetime().time();
        let time = Time::new(h, m, 0, 0).unwrap();

        if now < time {
            return prev_goal_val.clone();
        }
        prev_goal_val = goal_val
    }

    default
}
