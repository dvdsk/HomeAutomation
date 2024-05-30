pub mod lamps;
// pub mod mpd_control;

pub use lamps::Lighting;
// pub use mpd_control::Mpd;

use crate::input::jobs::Jobs;

#[derive(Debug, Clone)]
pub struct System {
    pub jobs: Jobs,
    pub lights: Lighting,
    // pub mpd: Mpd,
}

impl System {
    pub fn init(jobs: Jobs, hue_bridge_ip: String) -> Self {
        Self {
            jobs,
            lights: Lighting::start_init(hue_bridge_ip),
            // mpd: Mpd,
        }
    }
}
