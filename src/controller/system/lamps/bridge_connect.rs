use philipshue::bridge;
use philipshue::bridge::Bridge;
use philipshue::errors::{BridgeError::LinkButtonNotPressed, HueError, HueErrorKind};
use tracing::{info, error};

use super::Error;
use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;
use std::thread;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum RegisterError {
    #[error("Something went wrong registering the user inside the hue library")]
    HueError(philipshue::errors::HueError),
    #[error("Link button was not pressed within 25 seconds")]
    Timedout,
}

fn register(ip: &str) -> Result<String, RegisterError> {
    for _ in 0..5 {
        //try 5 times to connect
        match bridge::register_user(ip, "homeAutomationSys") {
            Ok(received_login) => {
                println!("Success, linked to bridge");
                info!("User registered: {}, on IP: {}", &received_login, ip);
                return Ok(received_login);
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
                println!("Unexpected error occurred: {}", e);
                return Err(RegisterError::HueError(e));
            }
        }
    }
    return Err(RegisterError::Timedout);
}

fn find_bridge_ip() -> Result<String, Error> {
    let mut discovered = bridge::discover().map_err(Error::Discovery)?;
    if discovered.len() == 0 {
        error!("No bridge found!");
        return Err(Error::NoBridgeFound);
    } else if discovered.len() > 1 {
        error!(
            "Found multiple hue bridges: {:?}, continuing with first one in list",
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

#[derive(Debug, thiserror::Error)]
pub enum SaveBridgeError {
    #[error("Could not create file {path:?} to store bridge logins in: {err}")]
    CreatingFile {
        path: &'static Path,
        err: std::io::Error,
    },
    #[error("Error writing bridge logins to file: {0}")]
    Writing(serde_yaml::Error),
}
fn update_saved_bridge_info(bridge_ip: &str, login: &str) -> Result<(), SaveBridgeError> {
    let path = Path::new("hueBridgeLogin");
    let file = File::create(&path).map_err(|err| SaveBridgeError::CreatingFile { err, path })?;
    serde_yaml::to_writer(file, &(bridge_ip, login)).map_err(SaveBridgeError::Writing)
}

pub fn get_bridge_and_status() -> Result<(Bridge, BTreeMap<usize, philipshue::hue::Light>), Error> {
    let Ok((ip, login)) = saved_bridge_info() else {
        let ip = find_bridge_ip()?;
        let login = register(&ip)?;
        update_saved_bridge_info(&ip, &login)?;

        let bridge = Bridge::new(&ip, &login);
        let lights_info = bridge.get_all_lights().map_err(Error::GettingLights)?;
        return Ok((bridge, lights_info));
    };

    let bridge = Bridge::new(&ip, &login);
    let res = bridge.get_all_lights();

    match res {
        Ok(lights_info) => Ok((bridge, lights_info)),
        Err(_) => {
            let ip = find_bridge_ip()?;
            let login = register(&ip)?;
            update_saved_bridge_info(&ip, &login)?;

            let bridge = Bridge::new(&ip, &login);
            let lights_info = bridge.get_all_lights().map_err(Error::GettingLights)?;
            Ok((bridge, lights_info))
        }
    }
}
