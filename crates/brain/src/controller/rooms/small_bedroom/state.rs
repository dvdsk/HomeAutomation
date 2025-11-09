use std::sync::Arc;
use std::time::Duration;

use color_eyre::Result;
use jiff::Zoned;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, Mutex};
use tokio::task::{self, JoinHandle};
use tokio::time::sleep;
use tracing::{trace, warn};

use super::audiocontrol::AudioController;
use super::{
    air_filtration_now, daylight_now, goal_temp_now, is_nap_time, NAP_TIME,
    OFF_DELAY,
};
use crate::controller::{Event, RestrictedSystem};

const MPD_IP: &str = "192.168.1.101";
const MPD_PORT: &str = "6600";

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) enum State {
    Sleep,
    SleepNoWakeup,
    Wakeup,
    #[default]
    Daylight,
    Override,
    DelayedOff,
    Nightlight,
}

#[dbstruct::dbstruct(db=sled)]
pub(super) struct Store {
    #[dbstruct(Default)]
    state: State,
    radiator_override: Option<Zoned>,
    pub(super) pm2_5: Option<(f32, Zoned)>,
}

#[derive(Clone)]
pub(super) struct Room {
    pub(super) store: Arc<Store>,
    system: RestrictedSystem,
    event_tx: broadcast::Sender<Event>,
    task_handle: Arc<Mutex<Option<JoinHandle<Result<()>>>>>,
    pub(super) audio_controller: Arc<Mutex<AudioController>>,
}

super::super::impl_open_or_wipe!(Store);

impl Room {
    pub(super) fn new(
        event_tx: broadcast::Sender<Event>,
        system: RestrictedSystem,
        db: sled::Tree,
    ) -> Result<Self> {
        let store = Arc::new(open_or_wipe(db)?);
        Ok(Self {
            store,
            system,
            event_tx,
            task_handle: Arc::new(Mutex::new(None)),
            audio_controller: Arc::new(Mutex::new(AudioController::new(
                MPD_IP, MPD_PORT,
            ))),
        })
    }

    pub(super) async fn update_radiator(&mut self) -> Result<()> {
        trace!("Updating radiator");
        // trace!(
        //     "Room radiator override state: {:?}",
        //     &self.radiator_override
        // );
        // if let Some(override_time) = &self.radiator_override {
        //     if crate::time::now()
        //         <= override_time
        //             .checked_add(RADIATOR_OVERRIDE_MINUTES.minute())
        //             .unwrap()
        //     {
        //         trace!("Override is set and not expired, do nothing");
        //         return;
        //     } else {
        //         warn!("Radiator override is expired, resetting");
        //     }
        // }
        let goal_temp = goal_temp_now();
        // trace!("Override is either not set or expired, set to goal temp: {goal_temp}");
        self.system.set_radiators_setpoint(goal_temp).await;
        self.store.radiator_override().set(None)?;

        Ok(())
    }

    #[allow(unused)]
    //TODO: fix radiator override
    pub(crate) fn start_radiator_override(&mut self) -> Result<()> {
        // Don't set if the radiator resends the manual state
        if self.store.radiator_override().is_none()? {
            warn!("Setting radiator override to now");
            let now = crate::time::now();
            self.store.radiator_override().set(Some(&now));
        }

        Ok(())
    }

    pub(super) async fn toggle_sleep_daylight(&mut self) -> Result<()> {
        let state = self.store.state().get()?;
        match state {
            State::Sleep | State::SleepNoWakeup => {
                self.set_daylight().await?;
            }
            State::Daylight
            | State::Wakeup
            | State::Override
            | State::DelayedOff
            | State::Nightlight => {
                self.set_sleep_immediate().await?;
            }
        }
        Ok(())
    }

    pub(super) async fn toggle_sleep_nightlight(&mut self) -> Result<()> {
        let state = self.store.state().get()?;
        match state {
            State::Sleep | State::SleepNoWakeup => {
                self.set_nightlight().await?;
            }
            State::Daylight
            | State::Wakeup
            | State::Override
            | State::DelayedOff
            | State::Nightlight => {
                self.set_sleep_immediate().await?;
            }
        }
        Ok(())
    }

    pub(super) async fn set_sleep_immediate(&mut self) -> Result<()> {
        self.system.all_lamps_off().await;
        self.set_state_cancel_prev(State::Sleep).await?;
        Ok(())
    }

    pub(super) async fn set_sleep_delayed(&mut self) -> Result<()> {
        self.set_state_cancel_prev(State::DelayedOff).await?;

        self.system.one_lamp_off("small_bedroom:bureau").await;
        self.system.one_lamp_off("small_bedroom:piano").await;

        let task_handle =
            task::spawn_local(self.clone().delayed_off_then_sleep());
        *self.task_handle.lock().await = Some(task_handle);
        Ok(())
    }

    pub(super) async fn set_wakeup(&mut self) -> Result<()> {
        if self.store.state().get()? == State::SleepNoWakeup {
            warn!("Ignoring wakeup because of override");
            return Ok(());
        }
        warn!("Starting wakeup");
        self.set_state_cancel_prev(State::Wakeup).await?;

        let task_handle =
            task::spawn_local(self.clone().run_wakeup_then_daylight());
        *self.task_handle.lock().await = Some(task_handle);
        Ok(())
    }

    pub(super) async fn set_daylight(&mut self) -> Result<()> {
        self.set_state_cancel_prev(State::Daylight).await?;
        self.all_lights_daylight().await?;
        self.system.all_lamps_on().await;
        Ok(())
    }

    pub(super) async fn set_nightlight(&mut self) -> Result<()> {
        self.set_state_cancel_prev(State::Nightlight).await?;
        self.system
            .one_lamp_ct("small_bedroom:ceiling", 1800, 0.1)
            .await;
        self.system.one_lamp_on("small_bedroom:ceiling").await;
        Ok(())
    }

    pub(super) async fn set_override(&mut self) -> Result<()> {
        self.set_state_cancel_prev(State::Override).await?;
        self.system.all_lamps_ct(2000, 1.0).await;
        self.system.all_lamps_on().await;
        Ok(())
    }

    async fn set_state_cancel_prev(&mut self, state: State) -> Result<()> {
        self.cancel_task().await;
        self.store.state().set(&state)?;
        let _ = self.event_tx.send(Event::StateChangeSB(state));
        Ok(())
    }

    async fn set_state(&mut self, state: State) -> Result<()> {
        self.store.state().set(&state)?;
        let _ = self.event_tx.send(Event::StateChangeSB(state));
        Ok(())
    }

    async fn cancel_task(&mut self) {
        if let Some(ref task) = *self.task_handle.lock().await {
            task.abort();
        }
    }

    async fn delayed_off_then_sleep(mut self) -> Result<()> {
        sleep(OFF_DELAY).await;
        self.system.all_lamps_off().await;
        self.set_state(State::Sleep).await?;

        if is_nap_time() {
            sleep(NAP_TIME).await;
            self.set_state(State::Daylight).await?;
            self.all_lights_daylight().await?;
            self.system.all_lamps_on().await;
        }
        Ok(())
    }

    // Returns result simply so the task handle when this is spawned as a task
    // is the same as for the other function
    async fn run_wakeup_then_daylight(mut self) -> Result<()> {
        const LIGHT_NAME: &str = "small_bedroom:ceiling";
        const RUNTIME_MINS: i32 = 5;
        const MUSIC_ON_AFTER_MINS: i32 = 5;

        const STEP_SIZE_SECS: i32 = 30;
        const N_STEPS: i32 = RUNTIME_MINS * 60 / STEP_SIZE_SECS;
        const MUSIC_STEP: i32 = MUSIC_ON_AFTER_MINS * 60 / STEP_SIZE_SECS;

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

            if step == MUSIC_STEP {
                let mut audio = self.audio_controller.lock().await;
                audio.start_wakeup_music().await;
            }
        }

        self.set_state(State::Daylight).await?;
        self.all_lights_daylight().await?;
        self.system.all_lamps_on().await;
        warn!("Wakeup done");

        Ok(())
    }

    // TODO: make private once updates are in job system
    pub(super) async fn all_lights_daylight(&mut self) -> Result<()> {
        if self.store.state().get()? == State::Daylight {
            let (new_ct, new_bri) = daylight_now();
            self.system.all_lamps_ct(new_ct, new_bri).await;
        }
        Ok(())
    }

    pub(crate) async fn update_airbox(&self) -> Result<()> {
        let Some(setting) = air_filtration_now(&self.store.pm2_5().get()?)
        else {
            return Ok(());
        };

        if let Ok(mut stream) = TcpStream::connect("192.168.1.103:4444").await {
            let message: u16 = 0xDD00 + setting;
            let _ = stream.write(&message.to_le_bytes()).await;
        }
        Ok(())
    }
}
