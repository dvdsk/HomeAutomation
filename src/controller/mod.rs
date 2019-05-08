use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::thread;

mod system;
use system::lamps::Lighting;

mod states;
mod commands;
use commands::handle_cmd;

#[derive(Copy, Clone)]
pub enum Command {
  LampsToggle,
  LampsDim,
  LampsDimmest,
  LampsEvening,
  LampsNight,
  LampsDay,
  LampsOff,
  LampsOn,

  ChangeState(State),
}

pub enum Event {
  Update,
  Command(Command),
  Stop,
}

#[derive(Copy, Clone)]
pub enum State {
  Normal,
	LightLoop,
  Other, //TODO GET OUT OF STATE, replace vec with small vec
}

pub struct Modifications { //change name to: alteration, deviation, overrides or something else?
  lighting: bool
  //Mpd,
  //Desk,
  //Alarm,
}

impl Default for Modifications {
	fn default() -> Self {
		Modifications {lighting: false}
	}
}

pub struct System {
	update_period: Duration,
	next_update: Instant,

	lights: Lighting,
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

pub fn start(rx: mpsc::Receiver<Event>) -> Result<(),()>{

	let mut system = System {
		update_period: Duration::from_secs(5),
		next_update: Instant::now()+Duration::from_secs(5),

		lights: Lighting::init()?,
		//mpd init?
	};

	thread::spawn(move || {
		let mut state = State::Normal; //initial state
		let mut mods = Modifications::default();
		// TODO guess best init state from statefile or lamps+mpd+time
	  
		
		loop {
			
			//wait for next update or an incoming event
			let start_recv = Instant::now();
			let time_till_update = saturating_duration_till(system.next_update);
			let event = if let Ok(event) =	rx.recv_timeout(time_till_update){
				event
			} else {
				system.next_update = Instant::now()+system.update_period;
				Event::Update
			};
			
			state = match (event, state) {
			    (Event::Update, State::Normal) => {states::normal::update(&mods, &mut system); State::Normal},


			    (Event::Command(cmd), state) => {handle_cmd(cmd, state, &mut mods, &mut system)},
			    (Event::Update, state) => {println!("default update funct"); state},
			    (Event::Stop, _) => break,
			};
		}
	});
	Ok(())
}

