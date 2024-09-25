use hueclient::Bridge;
use hueclient::HueError;
use tracing::{error, info};

use super::Error;
use std::fs::File;
use std::net::IpAddr;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum RegisterError {
    #[error("Something went wrong registering the user inside the hue library")]
    HueError(hueclient::HueError),
    #[error("Link button was not pressed within 25 seconds")]
    Timedout,
}

async fn register(ip: IpAddr) -> Result<Bridge, RegisterError> {
    for _ in 0..5 {
        //try 5 times to connect
        let bridge = Bridge::for_ip(ip);
        match bridge.register_user("ha-brain").await {
            Ok(bridge) => {
                println!("Success, linked to bridge");
                info!("User registered: {}, on IP: {ip:?}", bridge.username);
                return Ok(bridge);
            }
            Err(HueError::BridgeError { code: 101, .. }) => {
                println!("Please, press the link on the bridge. Retrying in 5 seconds");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
            Err(e) => {
                println!("Unexpected error occurred: {}", e);
                return Err(RegisterError::HueError(e));
            }
        }
    }
    Err(RegisterError::Timedout)
}

fn saved_bridge_info() -> Result<(String, String), ()> {
    let path = Path::new("hueBridgeLogin");
    match File::open(path) {
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
fn update_saved_bridge_info(bridge_ip: IpAddr, login: &str) -> Result<(), SaveBridgeError> {
    let path = Path::new("hueBridgeLogin");
    let file = File::create(path).map_err(|err| SaveBridgeError::CreatingFile { err, path })?;
    serde_yaml::to_writer(file, &(bridge_ip, login)).map_err(SaveBridgeError::Writing)
}

pub async fn get_bridge_and_status(
    ip: &str,
) -> Result<(Bridge, Vec<hueclient::IdentifiedLight>), Error> {
    let ip: IpAddr = ip.parse().expect("string should be an ip");
    let Ok((ip, login)) = saved_bridge_info() else {
        let bridge = register(ip).await?;
        update_saved_bridge_info(ip, &bridge.username)?;

        let lights_info = bridge
            .get_all_lights()
            .await
            .map_err(Error::GettingLights)?;
        return Ok((bridge, lights_info));
    };

    let ip: IpAddr = ip.parse().expect("string should be an ip");
    let bridge = Bridge::for_ip(ip).with_user(login);
    let res = bridge.get_all_lights().await;

    match res {
        Ok(lights_info) => Ok((bridge, lights_info)),
        Err(_) => {
            let bridge = register(ip).await?;
            update_saved_bridge_info(ip, &bridge.username)?;

            let lights_info = bridge
                .get_all_lights()
                .await
                .map_err(Error::GettingLights)?;
            Ok((bridge, lights_info))
        }
    }
}
