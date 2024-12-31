use std::net::IpAddr;

// pub mod mpd_control;
use zigbee_bridge::Controller;
// pub use mpd_control::Mpd;

use crate::input::jobs::Jobs;

#[derive(Debug, Clone)]
pub struct System {
    #[allow(dead_code)]
    pub jobs: Jobs,
    pub zigbee: Controller,
    // pub mpd: Mpd,
}

impl System {
    pub fn init(mqtt_ip: IpAddr, jobs: Jobs) -> Self {
        Self {
            jobs,
            zigbee: Controller::start_bridge(mqtt_ip, "brain"),
            // mpd: Mpd,
        }
    }
}
