use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::thread;

mod system;
use system::lamps::Lighting;

mod states;
use states::{ActiveState, RoomState};
mod commands;
use commands::{handle_cmd};
pub use commands::{Command, TargetState};

pub enum Event {
  Update,
	Alarm,
	Test,
  Command(Command),
  Stop,
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
		let mut mods = Modifications::default();
		// TODO guess best init state from statefile or lamps+mpd+time
	  
		let mut state = ActiveState::Normal(states::Normal::enter(&mut mods, &mut system)); //initial state

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
					//specific test code for normal state
			    (Event::Test, ActiveState::Normal(state)) => {dbg!("a test happend while in normal state"); ActiveState::Normal(state)},

					//general code that is the same for all functions, unless specific handlers exist above
			    (Event::Command(cmd), state) => {handle_cmd(cmd, state, &mut mods, &mut system)},
			    (Event::Update, state) => {state.update(&mut mods, &mut system)},	    
					(Event::Alarm, state) => {dbg!("ALARM ALARM"); state},
					(Event::Test, state) => {dbg!("a test happend"); state},
					
					(Event::Stop, _) => break,
			};
		}
	});
	Ok(())
}

