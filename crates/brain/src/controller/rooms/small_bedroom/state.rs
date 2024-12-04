use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::task::{self, JoinHandle};
use tokio::time::sleep;

use super::{daylight_now, OFF_DELAY};
use crate::controller::{Event, RestrictedSystem};

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub(crate) enum State {
    Sleep,
    Wakeup,
    Daylight,
    Override,
    DelayedOff,
}

#[derive(Clone)]
pub(super) struct Room {
    state: Arc<RwLock<State>>,
    system: RestrictedSystem,
    event_tx: broadcast::Sender<Event>,
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Room {
    pub(super) fn new(
        event_tx: broadcast::Sender<Event>,
        system: RestrictedSystem,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(State::Daylight)),
            system,
            event_tx,
            task_handle: Arc::new(Mutex::new(None)),
        }
    }

    pub(super) async fn to_sleep(&mut self) {
        self.to_state_cancel_prev(State::DelayedOff).await;

        self.system.one_lamp_off("small_bedroom:bureau").await;
        self.system.one_lamp_off("small_bedroom:piano").await;

        let task_handle = task::spawn(self.clone().delayed_off_then_sleep());
        *self.task_handle.lock().await = Some(task_handle);
    }

    pub(super) async fn to_wakeup(&mut self) {
        self.to_state_cancel_prev(State::Wakeup).await;

        let task_handle = task::spawn(self.clone().run_wakeup_then_daylight());
        *self.task_handle.lock().await = Some(task_handle);
    }

    pub(super) async fn to_daylight(&mut self) {
        self.to_state_cancel_prev(State::Daylight).await;
        self.all_lights_daylight().await;
        self.system.all_lamps_on().await;
    }

    pub(super) async fn to_override(&mut self) {
        self.to_state_cancel_prev(State::Override).await;
        self.system.all_lamps_ct(2000, 1.0).await;
        self.system.all_lamps_on().await;
    }

    async fn to_state_cancel_prev(&mut self, state: State) {
        self.cancel_task().await;
        *self.state.write().await = state.clone();
        let _ = self.event_tx.send(Event::StateChangeSB(state));
    }

    async fn cancel_task(&mut self) {
        if let Some(ref task) = *self.task_handle.lock().await {
            task.abort();
        }
    }

    async fn delayed_off_then_sleep(mut self) {
        sleep(OFF_DELAY).await;
        self.system.all_lamps_off().await;
        *self.state.write().await = State::Sleep;
    }

    async fn run_wakeup_then_daylight(mut self) {
        let light_name = "small_bedroom:piano";
        let bri = 1. / 254.;
        let ct = 2000;
        let bri_growth: f64 = 1.32;
        let ct_growth: f64 = 1.028;

        self.system.one_lamp_ct(light_name, ct, bri).await;
        // Make sure the light is the right ct and bri before turning it on
        sleep(Duration::from_secs(1)).await;
        self.system.one_lamp_on(light_name).await;

        for minute in 1..=20 {
            sleep(Duration::from_secs(60)).await;

            let new_bri = bri * bri_growth.powi(minute);
            let new_ct = (ct as f64 * ct_growth.powi(minute)).round() as usize;

            self.system.one_lamp_ct(light_name, new_ct, new_bri).await;
        }

        *self.state.write().await = State::Daylight;
        self.all_lights_daylight().await;
        self.system.all_lamps_on().await;
    }

    // TODO: make private once updates are in job system
    pub(super) async fn all_lights_daylight(&mut self) {
        if *self.state.read().await == State::Daylight {
            let (new_ct, new_bri) = daylight_now();
            self.system.all_lamps_ct(new_ct, new_bri).await;
        }
    }
}
