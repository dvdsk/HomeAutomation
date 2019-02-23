use std::sync::mpsc;
use std::time::Duration;
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
  Other, //TODO GET OUT OF STATE, replace vec with small vec
}

pub enum Modification { //change name to: alteration, deviation, overrides or something else?
  Lighting,
  //Mpd,
  //Desk,
  //Alarm,
}

pub struct System {
	lights: Lighting,
	//mpd
	//etc
}

pub fn start(rx: mpsc::Receiver<Event>) -> Result<(),()>{

	let mut system = System {
		lights: Lighting::init()?,

	};

	thread::spawn(move || {
		let mut state = State::Normal; //initial state
		let mut mods = Vec::new();
		// TODO guess best init state from statefile or lamps+mpd+time
	  loop {
			let event = rx.recv_timeout(Duration::from_secs(5)).unwrap_or(Event::Update);
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

