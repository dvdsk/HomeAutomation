use std::time::{Duration, Instant};
use std::thread;
use serde::{Serialize, Deserialize};

mod system;
use system::{Lighting};

mod environment;
use environment::Environment;

mod state;
use state::{RoomState};
mod commands;
use commands::{handle_cmd};

pub use commands::Command;
pub use state::State;
#[cfg(feature = "sensors_connected")]
use crate::input::sensors::SensorValue;
use crate::input::mpd_status::MpdStatus;
use crate::input::buttons::Button;
use crate::errors::Error;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Event {
  Update,
  Alarm,
  Test,
  #[cfg(feature = "sensors_connected")]
  Sensor(SensorValue),
  Command(Command),
  PressShort(Button),
  PressLong(Button),
}

pub struct Modifications { //change name to: alteration, deviation, overrides or something else?
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
		Modifications {lighting: false, mpd: false}
	}
}

pub struct System {
	update_period: Duration,
	next_update: Instant,

	lights: Lighting,
	mpd: MpdStatus
	//mpd

	//etc
}

fn saturating_duration_till(target: std::time::Instant) -> std::time::Duration{
	let now = Instant::now();
	if now < target {
		target - now
	} else {
		Duration::from_secs(0)
	}
}

pub fn start(rx: crossbeam_channel::Receiver<Event>, mpd_status: MpdStatus) -> Result<thread::JoinHandle<()>, Error>{

	let mut system = System {
		update_period: Duration::from_secs(5),
		next_update: Instant::now()+Duration::from_secs(5),

		lights: Lighting::init()?,
		mpd: mpd_status,
	};

	let handle = thread::spawn(move || {
		let mut mods = Modifications::default();
		let mut env = Environment::default();
		// TODO guess best init state from statefile or lamps+mpd+time
	  
		let mut state: Box<dyn RoomState> = Box::new(state::Normal::setup(&mut mods, &mut system)); //initial state
		let mut current_state = State::Normal;

		loop {
			//wait for next update or an incoming event
			let time_till_update = saturating_duration_till(system.next_update);
			let event = match rx.recv_timeout(time_till_update){
				Ok(event) => event,
				Err(error) => match error {
					crossbeam_channel::RecvTimeoutError::Timeout => {
						system.next_update = Instant::now()+system.update_period;
						Event::Update
					},
					crossbeam_channel::RecvTimeoutError::Disconnected => return (),
				}
			};
			
			//state changes may not return errors
			let next_state = match (event, current_state) {
				//specific test code for normal state
				(Event::Test, State::Normal) => {dbg!("a test happend while in normal state"); None},
				//general code that is the same for all functions, unless specific handlers exist above
				(Event::Command(cmd), _) => handle_cmd(cmd, &mut mods, &mut system),
				(Event::Update, _) => state.update(&mut mods, &mut system, &mut env),	    
				(Event::Alarm, _) => Some(State::WakeUp),
				(Event::Test, _) => {dbg!("a test happend"); None},
				
				// #[cfg(feature = "sensors_connected")]
				(Event::Sensor(_), _) => None,

				(Event::PressShort(button), _) => {
					let cmd = match button {
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
					handle_cmd(cmd, &mut mods, &mut system)
				}
				(Event::PressLong(button), _) => {
					match button {
						Button::LampLeft => Some(State::Silent),
						Button::LampMid => Some(State::Silent),
						Button::LampRight => Some(State::Silent),

						Button::DeskLeftMost => Some(State::Sleep),
						Button::DeskLeft => Some(State::Normal),
						Button::DeskRightMost => Some(State::LightLoop),
						_ => None,
					}
				}
			};

			if let Some(next_state) = next_state {
				//state.breakdown()
				state = change_state(next_state, &mut mods, &mut system);
				current_state = next_state;
			}
		}
	});
	Ok(handle)
}

fn change_state(next_state: State, mods: &mut Modifications, system: &mut System) -> Box<dyn RoomState> {

	match next_state {
		State::Normal => Box::new(state::Normal::setup(mods, system)),
		State::LightLoop => Box::new(state::LightLoop::setup(mods, system)),
		State::WakeUp => Box::new(state::WakeUp::setup(mods, system)),
		State::Sleep => Box::new(state::Sleep::setup(mods, system)),
		State::Silent => Box::new(state::Silent::setup(mods, system)),
	}
}