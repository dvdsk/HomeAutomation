use std::collections::BTreeMap;
use std::fmt::Debug;
use std::time::Duration;

use futures_concurrency::future::Race;
use futures_util::FutureExt;
use jiff::civil::Time;
use jiff::{ToSpan, Zoned};
use protocol::small_bedroom::{portable_button_panel, ButtonPanel};
use tokio::sync::broadcast;
use tokio::time::{sleep_until, Instant};
use tracing::{debug, trace};

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
                // trace!("Starting radiator override");
                // room.start_radiator_override();
            }
            Trigger::Event(RelevantEvent::Wakeup) => room.to_wakeup().await,
            Trigger::Event(RelevantEvent::Pm2_5(val)) => {
                room.pm2_5 = Some((val, crate::time::now()))
            }
            Trigger::ShouldUpdate => {
                room.update_airbox().await;
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

    // Dots1 short: to sleep -> lights off
    // Dots1 long: to nightlight always -> one lamp on
    //
    // Dots2 short: to nightlight at night, otherwise daylight -> lamp(s) on
    // Dots2 long: to sleep, wakeup off -> lights off
    match event {
        E::Button(B::BottomLeft(_)) => room.to_sleep_delayed().await,
        E::PortableButton(P::Dots1ShortRelease) => {
            room.to_sleep_immediate().await
        }
        E::Button(B::BottomMiddle(_))
        | E::PortableButton(P::Dots2ShortRelease) => {
            use crate::time;
            let now = time::now();
            match time(now.hour(), now.minute()) {
                // rust analyzer seems to think this illegal, its not
                T23_00.. | T0_00..T9_00 => room.to_nightlight().await,
                _ => room.to_daylight().await,
            }
        }
        E::Button(B::BOttomRight(_)) => room.to_override().await,
        E::PortableButton(P::Dots2LongRelease) => room.to_sleep_no_wakeup().await,
        E::PortableButton(P::Dots1LongRelease) => room.to_nightlight().await,
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
        ((10, 00), 20.0),
        ((11, 00), 20.5),
        ((12, 00), 21.0),
        ((20, 00), 20.5),
        ((21, 00), 20.0),
        ((21, 30), 19.0),
        ((22, 00), 18.0),
    ]);

    goal_now(goals, 18.0)
}

fn air_filtration_now(pm2_5_measurement: &Option<(f32, Zoned)>) -> Option<u16> {
    let pm2_5_expiration = 10.minutes();
    let goals =
        BTreeMap::from([((00, 00), 80), ((18, 00), 100), ((22, 30), 80)]);

    let default = goal_now(goals, 80);

    let Some((pm2_5, measured_time)) = pm2_5_measurement else {
        return Some(default);
    };

    let is_expired = measured_time.checked_add(pm2_5_expiration).unwrap()
        < crate::time::now();
    if is_expired {
        Some(default)
    } else {
        match pm2_5 {
            // 0.0..2.0 => Some(0),
            // // Don't change anything
            // 2.0..4.0 => None,
            _ => Some(default),
        }
    }
}

// TODO: move to jobs system and remove update trigger
pub(super) fn daylight_now() -> (usize, f64) {
    let goals = BTreeMap::from([
        ((00, 00), (1800, 0.5)),
        ((08, 00), (2000, 0.5)),
        ((09, 00), (3800, 1.0)),
        ((19, 00), (3600, 1.0)),
        ((19, 30), (3300, 1.0)),
        ((19, 45), (3000, 1.0)),
        ((20, 00), (2800, 1.0)),
        ((20, 15), (2500, 1.0)),
        ((20, 30), (2000, 1.0)),
        ((21, 00), (1900, 0.8)),
        ((21, 30), (1800, 0.5)),
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

    prev_goal_val
}
