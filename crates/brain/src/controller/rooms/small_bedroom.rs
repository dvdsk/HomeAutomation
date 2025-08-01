use std::collections::BTreeMap;
use std::fmt::Debug;
use std::time::Duration;

use audiocontrol::{AudioController, ForceRewind};
use futures_concurrency::future::Race;
use futures_util::FutureExt;
use jiff::civil::Time;
use jiff::{ToSpan, Zoned};
use protocol::small_bedroom::{portable_button_panel, ButtonPanel};
use tokio::sync::broadcast;
use tokio::time::{sleep_until, Instant};
use tracing::{info, trace};

use self::filter::{RelevantEvent, Trigger};
use self::state::Room;
pub(crate) use self::state::State;
use crate::controller::rooms::common::RecvFiltered;
use crate::controller::{Event, RestrictedSystem};
use crate::time;

mod audiocontrol;
mod filter;
mod state;

const UPDATE_INTERVAL: Duration = Duration::from_secs(5);
const OFF_DELAY: Duration = Duration::from_secs(60);
const WAKEUP_EXPIRATION: Duration = Duration::from_secs(1800);
const NAP_TIME: Duration = Duration::from_secs(30 * 60);

const fn time(hour: i8, minute: i8) -> f64 {
    hour as f64 + minute as f64 / 60.
}
const T0_00: f64 = time(0, 0);
const T10_00: f64 = time(10, 0);
const T21_00: f64 = time(21, 0);

pub async fn run(
    mut event_rx: broadcast::Receiver<Event>,
    event_tx: broadcast::Sender<Event>,
    system: RestrictedSystem,
) {
    let mut room = Room::new(event_tx, system.clone());
    let mut next_update = Instant::now() + UPDATE_INTERVAL;

    let res = system
        .system
        .jobs
        .remove_all_with_event(Event::WakeupSB)
        .await;
    trace!("Removing old SB wakeup jobs returned: {res:#?}");

    loop {
        let get_event = event_rx
            .recv_filter_mapped(filter::filter)
            .map(Trigger::Event);
        let tick = sleep_until(next_update).map(|_| Trigger::ShouldUpdate);

        let trigger = (get_event, tick).race().await;
        match trigger {
            Trigger::Event(
                event @ RelevantEvent::Button(_)
                | event @ RelevantEvent::PortableButton(_),
            ) => {
                handle_button(&mut room, event).await;
            }
            Trigger::Event(RelevantEvent::RadiatorOverride) => {
                // trace!("Starting radiator override");
                // room.start_radiator_override();
            }
            Trigger::Event(RelevantEvent::Wakeup) => room.set_wakeup().await,
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

async fn handle_button(room: &mut Room, event: RelevantEvent) {
    use portable_button_panel::Reading as P;
    use ButtonPanel as B;

    match event {
        event @ RelevantEvent::Button(button) => match button {
            B::TopLeft(_) | B::TopMiddle(_) | B::TopRight(_) => {
                handle_audio_button(room, event).await;
            }
            B::BottomLeft(_) | B::BottomMiddle(_) | B::BottomRight(_) => {
                handle_light_button(room, event).await;
            }
        },
        event @ RelevantEvent::PortableButton(button) => match button {
            P::PlayPause
            | P::TrackNext
            | P::TrackPrevious
            | P::VolumeUp
            | P::VolumeUpHold
            | P::VolumeDown
            | P::VolumeDownHold
            | P::Dots1LongRelease
            | P::Dots2LongRelease => {
                handle_audio_button(room, event).await;
            }
            P::Dots1ShortRelease | P::Dots2ShortRelease => {
                handle_light_button(room, event).await;
            }
            b => info!("Pressed unimplemented button: {b:?}"),
        },
        _ => unreachable!(),
    }
}

async fn handle_audio_button(room: &mut Room, button_event: RelevantEvent) {
    use audiocontrol::AudioMode as A;
    use portable_button_panel::Reading as P;
    use ButtonPanel as B;
    use RelevantEvent as E;

    let mut audio = room.audio_controller.lock().await;
    match (&audio.mode, button_event) {
        // Music, Singing, Meditation back (short) = previous
        (
            A::Music | A::Singing | A::Meditation,
            E::Button(B::TopLeft(press)),
        ) if !press.is_long() => audio.previous(),
        (
            A::Music | A::Singing | A::Meditation,
            E::PortableButton(P::TrackPrevious),
        ) => audio.previous(),
        // Podcast back (short) = rewind
        (A::Podcast, E::Button(B::TopLeft(press))) if !press.is_long() => {
            audio.rewind()
        }
        (A::Podcast, E::PortableButton(P::TrackPrevious)) => audio.rewind(),

        // Music, Singing, Meditation forward (short) = next
        (
            A::Music | A::Singing | A::Meditation,
            E::Button(B::TopRight(press)),
        ) if !press.is_long() => audio.next(),
        (
            A::Music | A::Singing | A::Meditation,
            E::PortableButton(P::TrackNext),
        ) => audio.next(),
        // Podcast forward (short) = skip
        (A::Podcast, E::Button(B::TopRight(press))) if !press.is_long() => {
            audio.skip()
        }
        (A::Podcast, E::PortableButton(P::TrackNext)) => audio.skip(),

        (_, E::Button(B::TopMiddle(press))) if !press.is_long() => {
            audio.toggle_playback()
        }

        (_, E::Button(B::TopLeft(press))) if press.is_long() => {
            audio.prev_playlist();
            audio.play(ForceRewind::No)
        }

        (_, E::Button(B::TopRight(press))) if press.is_long() => {
            audio.next_playlist();
            audio.play(ForceRewind::No)
        }
        (_, E::PortableButton(P::Dots1LongRelease)) => {
            audio.next_playlist();
            audio.play(ForceRewind::No)
        }

        (_, E::Button(B::TopMiddle(press))) if press.is_long() => {
            audio.next_mode();
            audio.play(ForceRewind::No)
        }
        (_, E::PortableButton(P::Dots2LongRelease)) => {
            audio.next_mode();
            audio.play(ForceRewind::No)
        }
        (_, E::PortableButton(P::PlayPause)) => {
            audio.toggle_playback();
        }
        (_, E::PortableButton(P::VolumeUp)) => audio.increase_volume(),
        (_, E::PortableButton(P::VolumeDown)) => audio.decrease_volume(),

        (_, b) => info!("Unrecognised button pressed: {b:?}"),
    }
}

async fn handle_light_button(room: &mut Room, event: RelevantEvent) {
    use portable_button_panel::Reading as P;
    use ButtonPanel as B;
    use RelevantEvent as E;

    // Dots1 short: toggle sleep/daylight
    //
    // Dots2 short: wakeup
    match event {
        // Light buttons
        E::Button(B::BottomLeft(_)) => room.set_sleep_delayed().await,
        E::PortableButton(P::Dots1ShortRelease) => {
            let now = crate::time::now();
            match time(now.hour(), now.minute()) {
                // rust analyzer seems to think this illegal, its not
                T21_00.. | T0_00..T10_00 => {
                    room.toggle_sleep_nightlight().await
                }
                _ => room.toggle_sleep_daylight().await,
            }
        }
        E::Button(B::BottomMiddle(_))
        | E::PortableButton(P::Dots2ShortRelease) => {
            room.set_wakeup().await;
        }
        E::Button(B::BottomRight(_)) => room.set_override().await,
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
        ((0, 00), 18.0),
        ((8, 30), 19.0),
        ((10, 00), 20.0),
        ((11, 00), 20.5),
        ((12, 00), 21.0),
        ((18, 00), 20.5),
        ((19, 00), 20.0),
        ((21, 00), 19.5),
        ((21, 30), 19.0),
        ((22, 00), 18.0),
    ]);

    goal_now(goals, 18.0)
}

fn air_filtration_now(pm2_5_measurement: &Option<(f32, Zoned)>) -> Option<u16> {
    let pm2_5_expiration = 10.minutes();
    let goals =
        BTreeMap::from([((00, 00), 80), ((18, 00), 80), ((22, 30), 80)]);

    let default = goal_now(goals, 80);

    let Some((pm2_5, measured_time)) = pm2_5_measurement else {
        return Some(default);
    };

    let is_expired = measured_time.checked_add(pm2_5_expiration).unwrap()
        < crate::time::now();
    if is_expired {
        Some(default)
    } else {
        // TODO: use pm 2.5 value again
        #[allow(clippy::match_single_binding)]
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
        ((0, 00), (1800, 0.5)),
        ((8, 00), (2000, 0.5)),
        ((9, 00), (3800, 1.0)),
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
