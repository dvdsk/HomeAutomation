use crossbeam_channel::RecvTimeoutError::*;
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::{Duration, Instant};

mod system;
use system::Lighting;

mod environment;
use environment::Environment;

mod state;
use state::RoomState;
mod commands;
use commands::handle_cmd;

use crate::errors::Error;
use crate::input::jobs::WakeUp;
use crate::input::mpd_status::MpdStatus;
pub use commands::Command;
use sensor_value::{Button, Press, SensorValue};
pub use state::{State, WakeUpStateError};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Event {
    Update,
    WakeUp,
    Test,
    Command(Command),
    Sensor(SensorValue),
}

pub struct Modifications {
    //change name to: alteration, deviation, overrides or something else?
    lighting: bool,
    mpd: bool,
    //Desk,
    //Alarm,
}

impl Modifications {
    fn reset(&mut self) {
        self.lighting = false;
        self.mpd = false;
    }
}

impl Default for Modifications {
    fn default() -> Self {
        Modifications {
            lighting: false,
            mpd: false,
        }
    }
}

pub struct System {
    update_period: Duration,
    next_update: Instant,
    lights: Lighting,
    mpd: MpdStatus,
    wakeup: WakeUp,
}

fn saturating_duration_till(target: std::time::Instant) -> std::time::Duration {
    let now = Instant::now();
    if now < target {
        target - now
    } else {
        Duration::from_secs(0)
    }
}

pub fn start(
    rx: crossbeam_channel::Receiver<Event>,
    mpd_status: MpdStatus,
    wakeup: WakeUp,
) -> Result<thread::JoinHandle<()>, Error> {
    let mut system = System {
        update_period: Duration::from_secs(5),
        next_update: Instant::now() + Duration::from_secs(5),
        lights: Lighting::init()?,
        mpd: mpd_status,
        wakeup,
    };

    let handle = thread::spawn(move || {
        let mut mods = Modifications::default();
        let mut env = Environment::default();
        // TODO guess best init state from statefile or lamps+mpd+time

        let mut state: Box<dyn RoomState> = state::Normal::setup(&mut mods, &mut system).unwrap(); //initial state
        let mut current_state = State::Normal;

        loop {
            //wait for next update or an incoming event
            let time_till_update = saturating_duration_till(system.next_update);
            let event = match rx.recv_timeout(time_till_update) {
                Ok(event) => event,
                Err(Timeout) => {
                    system.next_update = Instant::now() + system.update_period;
                    Event::Update
                }
                Err(Disconnected) => return,
            };

            let res = handle_event(
                event,
                current_state,
                &mut mods,
                &mut system,
                &mut env,
                &mut state,
            );
            match res {
                Err(e) => error!("Ran into an error handling an event: {:?}", e),
                Ok(Some(target_state)) => {
                    //should switch to another state
                    if let Ok(new_state) = change_state(target_state, &mut mods, &mut system) {
                        state = new_state;
                        current_state = target_state;
                    } //if state setup ran into an error we do not switch state
                }
                _ => (),
            }

            if let Event::Sensor(s) = event {
                env.update(s);
            }
        }
    });
    Ok(handle)
}

fn handle_event(
    event: Event,
    current_state: State,
    mods: &mut Modifications,
    system: &mut System,
    env: &mut Environment,
    state: &mut Box<dyn RoomState>,
) -> Result<Option<State>, Error> {
    //state changes may not return errors
    let next_state = match (event, current_state) {
        //specific test code for normal state
        (Event::Test, State::Normal) => {
            dbg!("a test happend while in normal state");
            None
        }
        //general code that is the same for all functions, unless specific handlers exist above
        (Event::Command(cmd), _) => handle_cmd(cmd, mods, system),
        (Event::Update, _) => state.update(mods, system, env)?,
        (Event::WakeUp, _) => {
            system.wakeup.reset()?;
            Some(State::WakeUp)
        }
        (Event::Test, _) => {
            dbg!("a test happend");
            None
        }

        (Event::Sensor(s), _) => handle_sensor(s, mods, system),
    };
    Ok(next_state)
}

fn handle_sensor(
    value: SensorValue,
    mods: &mut Modifications,
    system: &mut System,
) -> Option<State> {
    match value {
        SensorValue::ButtonPress(press) => handle_buttonpress(press, mods, system),
        // SensorValue::BathroomHum(b) => {dbg!(b); None}
        // SensorValue::BathroomTemp(t) => {dbg!(t); None}
        // SensorValue::MovementShower => {dbg!(b); None}
        // SensorValue::MovementToilet => {dbg!(b); None}
        _ => None, // for now we ignore the other sensor data
    }
}

fn handle_buttonpress(
    press: Press,
    mods: &mut Modifications,
    system: &mut System,
) -> Option<State> {
    const LONG_PRESS: u16 = 500; //ms
    if press.duration < LONG_PRESS {
        // millisec
        match press.button {
            Button::LampLeft => Some(State::Quiet),
            Button::LampMid => Some(State::Silent),
            Button::LampRight => Some(State::Off),

            Button::DeskLeftMost => Some(State::Sleep),
            Button::DeskLeft => Some(State::Normal),
            Button::DeskRight => Some(State::Quiet),
            Button::DeskRightMost => Some(State::Off),

            Button::DeskTop => handle_cmd(Command::MpdIncreaseVolume, mods, system),
            Button::DeskMid => handle_cmd(Command::MpdPause, mods, system),
            Button::DeskBottom => handle_cmd(Command::MpdDecreaseVolume, mods, system),
        }
    } else {
        let cmd = match press.button {
            Button::LampLeft => Command::LampsDim,
            Button::LampMid => Command::LampsDimmest,
            Button::LampRight => Command::LampsToggle,

            Button::DeskLeftMost => Command::LampsNight,
            Button::DeskLeft => Command::LampsEvening,
            Button::DeskRight => Command::LampsDay,
            Button::DeskRightMost => Command::LampsToggle,

            Button::DeskTop => Command::MpdIncreaseVolume,
            Button::DeskMid => Command::MpdPause,
            Button::DeskBottom => Command::MpdDecreaseVolume,
        };
        handle_cmd(cmd, mods, system)
    }
}

fn change_state(
    next_state: State,
    mods: &mut Modifications,
    system: &mut System,
) -> Result<Box<dyn RoomState>, Error> {
    let res = match &next_state {
        State::Normal => state::Normal::setup(mods, system),
        State::LightLoop => state::LightLoop::setup(mods, system),
        State::WakeUp => state::WakeUp::setup(mods, system),
        State::Sleep => state::Sleep::setup(mods, system),
        State::Silent => state::Silent::setup(mods, system),
        State::Off => state::Off::setup(mods, system),
    };

    if let Err(ref e) = &res {
        error!(
            "ran into error trying to switch to state: {:?}, error: {:?}",
            next_state, e
        );
    }
    res
}
