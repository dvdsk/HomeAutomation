use std::net::IpAddr;

// pub mod mpd_control;
use zigbee_bridge::lights;
// pub use mpd_control::Mpd;

use crate::input::jobs::Jobs;

#[derive(Debug, Clone)]
pub struct System {
    #[allow(dead_code)]
    pub jobs: Jobs,
    pub lights_new: lights::Controller,
    // pub mpd: Mpd,
}

impl System {
    pub fn init(mqtt_ip: IpAddr, jobs: Jobs) -> Self {
        Self {
            jobs,
            lights_new: lights::Controller::start_bridge(mqtt_ip),
            // mpd: Mpd,
        }
    }
}
