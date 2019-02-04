extern crate philipshue;
extern crate serde_yaml;

use philipshue::bridge;
use philipshue::bridge::Bridge;
use philipshue::hue::LightCommand;
use philipshue::errors::{HueError, HueErrorKind, BridgeError::{LinkButtonNotPressed, DeviceIsUnreachable}};

use std::thread;
use std::time::Duration;
use std::fs::File;
use std::path::Path;
use std::collections::{HashMap, BTreeMap};

use super::CommandServerState;
use super::actix_web::{HttpResponse, HttpRequest};

pub fn discover() -> Vec<String> {
    use philipshue::hue::Discovery;
    bridge::discover().unwrap().into_iter().map(Discovery::into_ip).collect()
}

fn register(ip: &str) -> Result<String, ()>{
	for _ in 0..5 {//try 5 times to connect
		match bridge::register_user(ip, "homeAutomationSys") {
			Ok(recieved_login) => {
					println!("Success, linked to brige");
			    info!("User registered: {}, on IP: {}", &recieved_login, ip);
			    return Ok(recieved_login);
			}
			Err(HueError(HueErrorKind::BridgeError { error: LinkButtonNotPressed, .. }, _)) => {
			    println!("Please, press the link on the bridge. Retrying in 5 seconds");
			    thread::sleep(Duration::from_secs(5));
			}
			Err(e) => {
			    println!("Unexpected error occured: {}", e);
			    return Err(());
			}
		}
	}
	return Err(());
}

pub fn toggle(req: &HttpRequest<CommandServerState>) -> HttpResponse {
	let mut lighting = req.state().lighting.write().unwrap();
	lighting.toggle();
	HttpResponse::Ok().finish()
}

pub fn dim(req: &HttpRequest<CommandServerState>) -> HttpResponse {
	let mut lighting = req.state().lighting.write().unwrap();
	lighting.set_all_to(50,500);
	HttpResponse::Ok().finish()
}

pub fn dimmest(req: &HttpRequest<CommandServerState>) -> HttpResponse {
	let mut lighting = req.state().lighting.write().unwrap();
	lighting.set_all_to(1,500);
	HttpResponse::Ok().finish()
}

pub fn normal(req: &HttpRequest<CommandServerState>) -> HttpResponse {
	let mut lighting = req.state().lighting.write().unwrap();
	lighting.set_all_to(254,220);
	HttpResponse::Ok().finish()
}

pub fn evening(req: &HttpRequest<CommandServerState>) -> HttpResponse {
	let mut lighting = req.state().lighting.write().unwrap();
	lighting.set_all_to(254,320);
	HttpResponse::Ok().finish()
}

pub fn night(req: &HttpRequest<CommandServerState>) -> HttpResponse {
	let mut lighting = req.state().lighting.write().unwrap();
	lighting.set_all_to(220,500);
	HttpResponse::Ok().finish()
}

fn find_bridge_ip() -> Result<String, ()> {
	let mut discovered = bridge::discover().unwrap();
	if discovered.len() == 0 {
		error!("No bridge found!");
		return Err(());
	} else if discovered.len() > 1 {
		error!("Found multiple hue bridges: {:?}, continueing with first one in list", discovered);
	}

	Ok(discovered.pop().unwrap().into_ip())
}

fn saved_bridge_info() -> Result<(String, String),()> {
	let path = Path::new("hueBridgeLogin");
	match File::open(&path){
		Err(e) => error!("find ip and login file, error: {}", e),
		Ok(f) => {
			match serde_yaml::from_reader(f){
				Ok((ip, login))=> return Ok((ip, login)),
				Err(e) => error!("could not read ip and login from file"),
			}
		},
	}
	Err(())
}

fn update_saved_bridge_info(bridge_ip: &str, login: &str) -> Result<(), ()> {
	let path = Path::new("hueBridgeLogin");
	match File::create(&path){
		Ok(f) => if let Err(e) = serde_yaml::to_writer(f,&(bridge_ip, login)){
			error!("could not write ip and login from file, error: {}", e);
		} else { return Ok(()) },
		Err(e) => {
			error!("cant open ip and login file, error: {}", e);
		},
	}
	Err(())
}

fn get_bridge_and_status() -> Result<(Bridge, BTreeMap<usize, philipshue::hue::Light>),()> {

	if let Ok((mut ip, mut login)) = saved_bridge_info(){
		let mut update_ip_or_login =	false;
		loop {
			let bridge = Bridge::new(&ip, &login);
			match bridge.get_all_lights(){
				Ok(lights_info) => {
					if update_ip_or_login {
						if update_saved_bridge_info(&ip, &login).is_err() {
							error!("Could not save new bridge ip and login, next run will fail without user intervention")
						}
					}
					return Ok((bridge, lights_info));
				},
				//cant find bridge on given ip
				Err(HueError(HueErrorKind::BridgeError{error: DeviceIsUnreachable, ..}, _)) => {
					ip = find_bridge_ip().map_err(|e| ())?;
					update_ip_or_login = true;
				},
				//cant register as button was not pressed in time
				Err(HueError(HueErrorKind::BridgeError{error: LinkButtonNotPressed, ..}, _)) => {
					login = register(&ip).map_err(|e| ())?;
					update_ip_or_login = true;
				},
				Err(e) => {
					error!("Unexpected error occured: {:?}", e);
		    	return Err(());
		    },
			}
		}
	} else {
		let ip = find_bridge_ip().map_err(|e| ())?;
		let login = register(&ip).map_err(|e| ())?;
		if update_saved_bridge_info(&ip, &login).is_err() {
			error!("Could not save new bridge ip and login, next run will fail without user intervention")
		};

		let bridge = Bridge::new(&ip, &login);
		let lights_info = bridge.get_all_lights().map_err(|e| ())?;
		return Ok((bridge, lights_info));
	}
}

//adaptation from philipshue LightState that adds some
//values and removes unused
struct Lamp {
  pub on: bool,
  pub bri: u8,
  pub hue: Option<u16>,
  pub sat: Option<u8>,
  pub xy: Option<(f32, f32)>,
  pub ct: Option<u16>,
  pub reachable: bool,
}

pub struct Lighting {
	bridge: Bridge,
	//local cache of current state (used for toggle)
	lamps: HashMap<usize, Lamp>,
}

impl From<&philipshue::hue::LightState> for Lamp{
	fn from(state: &philipshue::hue::LightState) -> Self {
		Lamp{
			on: state.on,
			bri: state.bri,
			hue: state.hue,
			sat: state.sat,
			xy: state.xy,
			ct: state.ct,
			reachable: state.reachable,
		}
	}
}

impl Lighting {

	pub fn init() -> Result<Self, ()> {
		let (bridge, lights_info) = get_bridge_and_status()?;
		let lamps: HashMap<usize, Lamp> = lights_info.iter().map(|(id,light)| (*id, Lamp::from(&light.state))).collect();

		Ok(Self {bridge, lamps})
	}

	//how to deal with errors?
	fn toggle(&mut self) -> Result<(),()>{
		let numb_on: u8 = self.lamps.values().map(|lamp| lamp.on as u8).sum();
		let numb_off =	self.lamps.len() as u8 - numb_on;

		//group ID 0 is a special group containing all lights known to the bridge
		if numb_on > numb_off {
			let command = LightCommand::off(LightCommand::default() );
			self.bridge.set_group_state(0, &command).map_err(|x| ())?;
			self.lamps.values_mut().for_each(|lamp| lamp.on = false);
		} else {
			let command = LightCommand::on(LightCommand::default() );
			self.bridge.set_group_state(0, &command).map_err(|x| ())?;
			self.lamps.values_mut().for_each(|lamp| lamp.on = true);
		}

		Ok(())
	}

	//how to deal with errors?
	fn set_all_to(&mut self, bri: u8, ct: u16) -> Result<(),()>{
		let command = LightCommand::default();
		let command = command.on();
		let command = command.with_bri(bri);
		let command = command.with_ct(ct);
		self.bridge.set_group_state(0, &command).map_err(|x| ())?;
		self.lamps.values_mut().for_each(|lamp| {lamp.bri =bri; lamp.ct =Some(ct)});

		Ok(())
	}

}
