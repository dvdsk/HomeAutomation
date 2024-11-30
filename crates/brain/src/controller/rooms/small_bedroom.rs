use std::sync::Arc;
use std::time::Duration;

use futures_concurrency::future::Race;
use futures_util::FutureExt;
use protocol::small_bedroom::ButtonPanel;
use protocol::{small_bedroom, Reading};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, Mutex};
use tokio::task;
use tokio::time::{sleep, sleep_until, Instant};
use tracing::warn;
use zigbee_bridge::lights::{denormalize, kelvin_to_mired};

use crate::controller::{Event, RestrictedSystem};

#[derive(PartialEq, Eq, Debug)]
enum State {
    Sleep,
    Wakeup,
    Normal,
    Override,
}

const UPDATE_INTERVAL: Duration = Duration::from_secs(5);
const OFF_DELAY: Duration = Duration::from_secs(60);

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
            let event = match self.recv().await {
                Ok(event) => event,
                Err(RecvError::Lagged(n)) => {
                    warn!("SB missed {n} events");
                    continue;
                }
                Err(err) => panic!("{err}"),
            };
            if let Some(relevant) = filter_map(event) {
                return relevant;
            }
        }
    }
}

#[derive(Debug)]
enum RelevantEvent {
    Button(ButtonPanel),
    Wakeup,
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
    let room_state = Arc::new(Mutex::new(State::Normal));
    let mut next_update = Instant::now() + UPDATE_INTERVAL;
    loop {
        let get_event = event_rx
            .recv_filter_mapped(event_filter)
            .map(Trigger::Event);
        let tick = sleep_until(next_update).map(|_| Trigger::ShouldUpdate);

        let trigger = (get_event, tick).race().await;
        match trigger {
            Trigger::Event(RelevantEvent::Button(button)) => {
                handle_buttonpress(system.clone(), room_state.clone(), button)
                    .await;
            }
            Trigger::Event(RelevantEvent::Wakeup) => {
                task::spawn(run_wakeup(system.clone(), room_state.clone()));
            }
            Trigger::ShouldUpdate => {
                set_time_color(&mut system, room_state.clone()).await;
                next_update = Instant::now() + UPDATE_INTERVAL;
            }
        }
    }
}

async fn run_wakeup(
    mut system: RestrictedSystem,
    room_state: Arc<Mutex<State>>,
) {
    {
        *room_state.lock().await = State::Wakeup;
    }

    let light_name = "small_bedroom:piano";
    let bri = 1;
    let ct = 2000;
    let bri_growth: f64 = 1.32;
    let ct_growth: f64 = 1.028;

    system
        .one_lamp_ct(light_name, kelvin_to_mired(ct).try_into().unwrap(), bri)
        .await;
    // Make sure the light is the right ct and bri before turning it on
    sleep(Duration::from_secs(1)).await;
    system.one_lamp_on(light_name).await;

    for minute in 1..=20 {
        sleep(Duration::from_secs(1)).await;
        // room state has changed, thus wakeup should be aborted
        if *room_state.lock().await != State::Wakeup {
            return;
        }

        let new_bri = ((bri as f64) * bri_growth.powi(minute)).round() as u8;
        let new_ct = (ct as f64 * ct_growth.powi(minute)).round() as usize;

        system
            .one_lamp_ct(
                light_name,
                kelvin_to_mired(new_ct).try_into().unwrap(),
                new_bri,
            )
            .await;
    }

    *room_state.lock().await = State::Normal;
}

async fn delayed_off(
    mut system: RestrictedSystem,
    room_state: Arc<Mutex<State>>,
) {
    sleep(OFF_DELAY).await;
    // room state has changed, thus off should be aborted
    if *room_state.lock().await != State::Sleep {
        return;
    }
    system.all_lamps_off().await;
}

async fn handle_buttonpress(
    mut system: RestrictedSystem,
    room_state: Arc<Mutex<State>>,
    button: ButtonPanel,
) {
    dbg!(button);
    match button {
        ButtonPanel::BottomLeft(_) => {
            {
                *room_state.lock().await = State::Sleep;
            }
            system.one_lamp_off("small_bedroom:bureau").await;
            system.one_lamp_off("small_bedroom:piano").await;
            task::spawn(delayed_off(system.clone(), room_state.clone()));
        }
        ButtonPanel::BottomMiddle(_) => {
            {
                *room_state.lock().await = State::Normal;
            }
            system.all_lamps_on().await;
            set_time_color(&mut system, room_state).await;
        }
        ButtonPanel::BOttomRight(_) => {
            task::spawn(run_wakeup(system.clone(), room_state.clone()));
            // *room_state = State::Override;
            // system.all_lamps_on().await;
            // system.all_lamps_ct(2000, 254).await;
        }
        _ => (),
    }
}

async fn set_time_color(
    system: &mut RestrictedSystem,
    room_state: Arc<Mutex<State>>,
) {
    if *room_state.lock().await == State::Normal {
        let (new_ct, new_bri) = optimal_ct_bri();
        system.all_lamps_ct(new_ct, new_bri).await;
    }
}

const fn time(hour: i8, minute: i8) -> f64 {
    hour as f64 + minute as f64 / 60.
}

pub(crate) fn optimal_ct_bri() -> (u16, u8) {
    let now = crate::time::now();
    const T0_00: f64 = time(0, 0);
    const T8_00: f64 = time(8, 0);
    const T9_00: f64 = time(9, 0);
    const T17_00: f64 = time(17, 0);
    const T20_30: f64 = time(20, 30);
    const T21_30: f64 = time(21, 30);
    const T22_00: f64 = time(22, 0);

    let (temp, bri) = match time(now.hour(), now.minute()) {
        T8_00..T9_00 => (2000, 0.5),
        T9_00..T17_00 => (3500, 1.0),
        T17_00..T20_30 => (2300, 1.0),
        T20_30..T21_30 => (2000, 0.7),
        T21_30..T22_00 => (1900, 0.4),
        T22_00.. | T0_00..T8_00 => (1800, 0.1),
        _ => (2300, 1.0),
    };
    (kelvin_to_mired(temp).try_into().unwrap(), denormalize(bri))
}
