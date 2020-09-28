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
use crate::input::mpd_status::MpdStatus;
use crate::errors::Error;
use crate::input::sensors::Button;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Event {
  Update,
  Alarm,
  Test,
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

		let mut state: Box<dyn RoomState> = state::Normal::setup(&mut mods, &mut system).unwrap(); //initial state
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
			
			match handle_event(event, current_state, &mut mods, 
				&mut system, &mut env, &mut state){
				
				Err(e) => error!("Ran into an error handling an event: {:?}", e),
				Ok(Some(target_state)) => {//should switch to another state
					if let Ok(new_state) = change_state(target_state, &mut mods, &mut system){
						state = new_state;
						current_state = target_state;
					} //if state setup ran into an error we do not switch state
				},
				_ => (),
			}
		}
	});
	Ok(handle)
}

fn handle_event(event: Event, current_state: State, mods: &mut Modifications, 
	system: &mut System, env: &mut Environment, state: &mut Box<dyn RoomState>) 
	-> Result<Option<State>, Error> {

		//state changes may not return errors
	let next_state = match (event, current_state) {
		//specific test code for normal state
		(Event::Test, State::Normal) => {dbg!("a test happend while in normal state"); None},
		//general code that is the same for all functions, unless specific handlers exist above
		(Event::Command(cmd), _) => handle_cmd(cmd, mods, system),
		(Event::Update, _) => state.update(mods, system, env)?,	    
		(Event::Alarm, _) => Some(State::WakeUp),
		(Event::Test, _) => {dbg!("a test happend"); None},
		
		//(Event::Sensor(_), _) => None,

		(Event::PressShort(button), _) => {
			match button {
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

				_ => None,
			}
		}
		(Event::PressLong(button), _) => {
			let cmd = match button {
				Button::LampLeft => Command::LampsDim,
				Button::LampMid => Command::LampsDimmest,
				Button::LampRight => Command::LampsToggle,
			
				Button::DeskLeftMost => Command::LampsNight,
				Button::DeskLeft => Command::LampsEvening,
				Button::DeskRight => Command::LampsDay,
				Button::DeskRightMost => Command::LampsToggle,

				_ => return Ok(None),
			};
			handle_cmd(cmd, mods, system)
		}
	};
	Ok(next_state)
}

fn change_state(next_state: State, mods: &mut Modifications, system: &mut System) -> Result<Box<dyn RoomState>, Error> {

	let res = match &next_state {
		State::Normal => state::Normal::setup(mods, system),
		State::LightLoop => state::LightLoop::setup(mods, system),
		State::WakeUp => state::WakeUp::setup(mods, system),
		State::Sleep => state::Sleep::setup(mods, system),
		State::Silent => state::Silent::setup(mods, system),
		State::Quiet => state::Quiet::setup(mods, system),
		State::Off => state::Off::setup(mods, system),
	};

	if let Err(ref e) = &res {
		error!("ran into error trying to switch to state: {:?}, error: {:?}", 
			next_state,e);
	}

	res
}