extern crate philipshue;
extern crate serde_yaml;

use philipshue::bridge;
use philipshue::bridge::Bridge;
use philipshue::errors::{
    BridgeError::{DeviceIsUnreachable, LinkButtonNotPressed},
    HueError, HueErrorKind,
};
use philipshue::hue::LightCommand;

use crate::errors::Error;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::path::Path;
use std::thread;
use std::time::Duration;

fn register(ip: &str) -> Result<String, ()> {
    for _ in 0..5 {
        //try 5 times to connect
        match bridge::register_user(ip, "homeAutomationSys") {
            Ok(recieved_login) => {
                println!("Success, linked to brige");
                info!("User registered: {}, on IP: {}", &recieved_login, ip);
                return Ok(recieved_login);
            }
            Err(HueError(
                HueErrorKind::BridgeError {
                    error: LinkButtonNotPressed,
                    ..
                },
                _,
            )) => {
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

fn find_bridge_ip() -> Result<String, Error> {
    let mut discovered = bridge::discover()?;
    if discovered.len() == 0 {
        error!("No bridge found!");
        return Err(Error::Lamps(HueError::from_kind(HueErrorKind::Msg(
            "no bridge found".into(),
        ))));
    } else if discovered.len() > 1 {
        error!(
            "Found multiple hue bridges: {:?}, continueing with first one in list",
            discovered
        );
    }

    Ok(discovered.pop().unwrap().into_ip())
}

fn saved_bridge_info() -> Result<(String, String), ()> {
    let path = Path::new("hueBridgeLogin");
    match File::open(&path) {
        Err(e) => error!("find ip and login file, error: {}", e),
        Ok(f) => match serde_yaml::from_reader(f) {
            Ok((ip, login)) => return Ok((ip, login)),
            Err(_) => error!("could not read ip and login from file"),
        },
    }
    Err(())
}

fn update_saved_bridge_info(bridge_ip: &str, login: &str) -> Result<(), ()> {
    let path = Path::new("hueBridgeLogin");
    match File::create(&path) {
        Ok(f) => {
            if let Err(e) = serde_yaml::to_writer(f, &(bridge_ip, login)) {
                error!("could not write ip and login from file, error: {}", e);
            } else {
                return Ok(());
            }
        }
        Err(e) => {
            error!("cant open ip and login file, error: {}", e);
        }
    }
    Err(())
}

fn get_bridge_and_status() -> Result<(Bridge, BTreeMap<usize, philipshue::hue::Light>), Error> {
    if let Ok((mut ip, mut login)) = saved_bridge_info() {
        let mut update_ip_or_login = false;
        loop {
            let bridge = Bridge::new(&ip, &login);
            match bridge.get_all_lights() {
                Ok(lights_info) => {
                    if update_ip_or_login {
                        if update_saved_bridge_info(&ip, &login).is_err() {
                            error!("Could not save new bridge ip and login, next run will fail without user intervention")
                        }
                    }
                    return Ok((bridge, lights_info));
                }
                //cant find bridge on given ip
                Err(HueError(
                    HueErrorKind::BridgeError {
                        error: DeviceIsUnreachable,
                        ..
                    },
                    _,
                )) => {
                    ip = find_bridge_ip()?;
                    update_ip_or_login = true;
                }
                //cant register as button was not pressed in time
                Err(HueError(
                    HueErrorKind::BridgeError {
                        error: LinkButtonNotPressed,
                        ..
                    },
                    _,
                )) => {
                    login = register(&ip)?;
                    update_ip_or_login = true;
                }
                Err(e) => {
                    error!("Unexpected error occured: {:?}", e);
                    return Err(e.into());
                }
            }
        }
    } else {
        let ip = find_bridge_ip()?;
        let login = register(&ip)?;
        if update_saved_bridge_info(&ip, &login).is_err() {
            error!("Could not save new bridge ip and login, next run will fail without user intervention")
        };

        let bridge = Bridge::new(&ip, &login);
        let lights_info = bridge.get_all_lights()?;
        return Ok((bridge, lights_info));
    }
}

//adaptation from philipshue LightState that adds some
//values and removes unused
#[derive(Debug)]
pub struct Lamp {
    pub on: bool,
    pub bri: u8,
    pub hue: Option<u16>,
    pub sat: Option<u8>,
    pub xy: Option<(f32, f32)>,
    pub ct: Option<u16>,
    pub reachable: bool,
}

pub struct Lighting {
    pub bridge: Bridge,
    //local cache of current state (used for toggle)
    pub lamps: HashMap<usize, Lamp>,
}

impl From<&philipshue::hue::LightState> for Lamp {
    fn from(state: &philipshue::hue::LightState) -> Self {
        Lamp {
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
    pub fn init() -> Result<Self, Error> {
        let (bridge, lights_info) = get_bridge_and_status()?;
        let lamps: HashMap<usize, Lamp> = lights_info
            .iter()
            .map(|(id, light)| (*id, Lamp::from(&light.state)))
            .collect();

        Ok(Self { bridge, lamps })
    }

    #[allow(unused)]
    pub fn print_info(&self) {
        dbg!(&self.lamps);
    }

    pub fn numb_on(&self) -> u8 {
        self.lamps.values().map(|lamp| lamp.on as u8).sum()
    }

    //how to deal with errors?
    pub fn toggle(&mut self) -> Result<(), Error> {
        let numb_on: u8 = self.numb_on();
        let numb_off = self.lamps.len() as u8 - numb_on;

        //group ID 0 is a special group containing all lights known to the bridge
        if numb_on > numb_off {
            let command = LightCommand::off(LightCommand::default());
            self.bridge.set_group_state(0, &command)?;
            self.lamps.values_mut().for_each(|lamp| lamp.on = false);
        } else {
            let command = LightCommand::on(LightCommand::default());
            self.bridge.set_group_state(0, &command)?;
            self.lamps.values_mut().for_each(|lamp| lamp.on = true);
        }

        Ok(())
    }

    pub fn all_off(&mut self) -> Result<(), Error> {
        let command = LightCommand::off(LightCommand::default());
        self.bridge.set_group_state(0, &command)?;
        self.lamps.values_mut().for_each(|lamp| lamp.on = false);
        Ok(())
    }

    pub fn all_on(&mut self) -> Result<(), Error> {
        let command = LightCommand::on(LightCommand::default());
        self.bridge.set_group_state(0, &command)?;
        self.lamps.values_mut().for_each(|lamp| lamp.on = false);
        Ok(())
    }

    pub fn single_off(&mut self, lamp_id: usize) -> Result<(), Error> {
        let command = LightCommand::off(LightCommand::default());
        self.bridge.set_light_state(lamp_id, &command)?;
        self.lamps.get_mut(&lamp_id).unwrap().on = true;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn single_on(&mut self, lamp_id: usize) -> Result<(), Error> {
        let command = LightCommand::on(LightCommand::default());
        self.bridge.set_light_state(lamp_id, &command)?;
        self.lamps.get_mut(&lamp_id).unwrap().on = true;
        Ok(())
    }

    //how to deal with errors?
    pub fn set_all_ct(&mut self, bri: u8, ct: u16) -> Result<(), Error> {
        let command = LightCommand::default();
        //let command = command.on();
        let command = command.with_bri(bri);
        let command = command.with_ct(ct);
        self.bridge.set_group_state(0, &command)?;
        self.lamps.values_mut().for_each(|lamp| {
            lamp.bri = bri;
            lamp.ct = Some(ct);
            lamp.on = true
        });

        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_all_xy(&mut self, bri: u8, xy: (f32, f32)) -> Result<(), Error> {
        let command = LightCommand::default();
        let command = command.on();
        let command = command.with_bri(bri);
        let command = command.with_xy(xy);
        self.bridge.set_group_state(0, &command)?;
        self.lamps.values_mut().for_each(|lamp| {
            lamp.bri = bri;
            lamp.xy = Some(xy);
            lamp.on = true
        });

        Ok(())
    }

    pub fn set_all_rgb(&mut self, bri: u8, rgb: (f32, f32, f32)) -> Result<(), Error> {
        let xy = xy_from_rgb(rgb);

        let command = LightCommand::default();
        let command = command.on();
        let command = command.with_bri(bri);
        let command = command.with_xy(xy);
        self.bridge.set_group_state(0, &command)?;
        self.lamps.values_mut().for_each(|lamp| {
            lamp.bri = bri;
            lamp.xy = Some(xy);
            lamp.on = true
        });

        Ok(())
    }
}

fn gamma_correct(mut x: f32) -> f32 {
    if x > 0.04045 {
        x = (x + 0.055) / (1f32 + 0.055);
        x.powf(2.4)
    } else {
        x / 12.92
    }
}

//r,g,b between 0 and one
//https://gist.github.com/popcorn245/30afa0f98eea1c2fd34d
fn xy_from_rgb(rgb: (f32, f32, f32)) -> (f32, f32) {
    let (r, g, b) = rgb;
    let r = gamma_correct(r);
    let g = gamma_correct(g);
    let b = gamma_correct(b);

    let xyz_x = r * 0.649926 + g * 0.103455 + b * 0.197109;
    let xyz_y = r * 0.234327 + g * 0.743075 + b * 0.022598;
    let xyz_z = g * 0.053077 + b * 1.035763;

    let hue_x = xyz_x / (xyz_x + xyz_y + xyz_z);
    let hue_y = xyz_y / (xyz_x + xyz_y + xyz_z);

    //TODO color gamut triangle stuff for finding closest valid value

    (hue_x, hue_y)
}
