use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::task::{self, JoinHandle};
use tokio::time::sleep;
use tracing::warn;

use super::{daylight_now, OFF_DELAY};
use crate::controller::{Event, RestrictedSystem};

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub(crate) enum State {
    Sleep,
    Wakeup,
    Daylight,
    Override,
    DelayedOff,
    Nightlight,
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
        warn!("Starting wakeup");
        self.to_state_cancel_prev(State::Wakeup).await;

        let task_handle = task::spawn(self.clone().run_wakeup_then_daylight());
        *self.task_handle.lock().await = Some(task_handle);
    }

    pub(super) async fn to_daylight(&mut self) {
        self.to_state_cancel_prev(State::Daylight).await;
        self.all_lights_daylight().await;
        self.system.all_lamps_on().await;
    }

    pub(super) async fn to_nightlight(&mut self) {
        self.to_state_cancel_prev(State::Nightlight).await;
        self.system.one_lamp_ct("small_bedroom:ceiling", 1800, 0.1).await;
        self.system.one_lamp_on("small_bedroom:ceiling").await;
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

    async fn to_state(&mut self, state: State) {
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
        self.to_state(State::Sleep).await;
    }

    async fn run_wakeup_then_daylight(mut self) {
        const LIGHT_NAME: &str = "small_bedroom:piano";
        const RUNTIME_MINS: i32 = 7;
        const STEP_SIZE_SECS: i32 = 30;
        const N_STEPS: i32 = RUNTIME_MINS / STEP_SIZE_SECS;

        const START_BRI: f64 = 1. / 254.;
        const START_CT: usize = 2000;
        const END_BRI: f64 = 1.0;
        const END_CT: usize = 3500;

        let bri_growth = (END_BRI / START_BRI).powf(1. / N_STEPS as f64);
        let ct_growth =
            (END_CT as f64 / START_CT as f64).powf(1. / N_STEPS as f64);

        self.system
            .one_lamp_ct(LIGHT_NAME, START_CT, START_BRI)
            .await;
        self.system.one_lamp_on(LIGHT_NAME).await;
        warn!("Wakeup lamp should be on now");

        for step in 1..=N_STEPS {
            sleep(Duration::from_secs(
                STEP_SIZE_SECS.try_into().expect("Should be positive"),
            ))
            .await;

            let new_bri = START_BRI * bri_growth.powi(step);
            let new_ct =
                (START_CT as f64 * ct_growth.powi(step)).round() as usize;

            self.system.one_lamp_ct(LIGHT_NAME, new_ct, new_bri).await;
        }

        self.to_state(State::Daylight).await;
        self.all_lights_daylight().await;
        self.system.all_lamps_on().await;
        warn!("Wakeup done");
    }

    // TODO: make private once updates are in job system
    pub(super) async fn all_lights_daylight(&mut self) {
        if *self.state.read().await == State::Daylight {
            let (new_ct, new_bri) = daylight_now();
            self.system.all_lamps_ct(new_ct, new_bri).await;
        }
    }
}
