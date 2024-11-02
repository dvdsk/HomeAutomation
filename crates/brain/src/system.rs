pub mod lamps;
// pub mod mpd_control;

pub use lamps::Lighting;
use zigbee_bridge::lights;
// pub use mpd_control::Mpd;

use crate::input::jobs::Jobs;

#[derive(Debug, Clone)]
pub struct System {
    #[allow(dead_code)]
    pub jobs: Jobs,
    pub lights: Lighting,
    pub lights_new: lights::Controller,
    // pub mpd: Mpd,
}

impl System {
    pub fn init(jobs: Jobs, hue_bridge_ip: String) -> Self {
        Self {
            jobs,
            lights: Lighting::start_init(hue_bridge_ip),
            lights_new: lights::Controller::start_bridge(),
            // mpd: Mpd,
        }
    }
}
