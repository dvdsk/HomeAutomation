#![allow(clippy::missing_panics_doc)]
use std::net::IpAddr;

use tokio::sync::mpsc;
use tracing::trace;

pub(crate) use lamp::Model;

mod cached_bridge;
mod conversion;
pub(crate) mod lamp;
mod parse_state;

#[derive(Debug, Clone)]
pub struct Controller {
    change_sender: mpsc::UnboundedSender<(String, lamp::LampProperty)>,
}

impl Controller {
    #[must_use]
    pub fn start_bridge(mqtt_ip: IpAddr) -> Self {
        let (change_sender, change_receiver) = mpsc::unbounded_channel();

        let run_bridge = cached_bridge::run(mqtt_ip, change_receiver);
        trace!("Spawning zigbee bridge task");
        tokio::task::spawn(run_bridge);

        Self { change_sender }
    }

    pub fn set_on(&self, friendly_name: &str) {
        self.change_sender
            .send((friendly_name.to_string(), lamp::LampProperty::On(true)))
            .expect("Sender should never be dropped");
    }

    pub fn set_off(&self, friendly_name: &str) {
        self.change_sender
            .send((friendly_name.to_string(), lamp::LampProperty::On(false)))
            .expect("Sender should never be dropped");
    }

    /// Brightness from 0 to 1
    pub fn set_brightness(&self, friendly_name: &str, brightness: f64) {
        self.change_sender
            .send((
                friendly_name.to_string(),
                lamp::LampProperty::Brightness(brightness),
            ))
            .expect("Sender should never be dropped");
    }

    pub fn set_color_temp(&self, friendly_name: &str, kelvin: usize) {
        self.change_sender
            .send((
                friendly_name.to_string(),
                lamp::LampProperty::ColorTempK(kelvin),
            ))
            .expect("Sender should never be dropped");
    }

    pub fn set_color_xy(&self, friendly_name: &str, xy: (f64, f64)) {
        self.change_sender
            .send((friendly_name.to_string(), lamp::LampProperty::ColorXY(xy)))
            .expect("Sender should never be dropped");
    }
}
