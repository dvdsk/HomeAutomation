#![allow(clippy::missing_panics_doc)]
use std::net::IpAddr;

use tokio::sync::mpsc;
use tracing::trace;

pub(crate) use lamp::Model;

use self::lamp::LampProperty;

mod cached_bridge;
mod conversion;
mod device;
pub(crate) mod lamp;
mod parse_state;
#[cfg(test)]
mod tests;

const MQTT_PORT: u16 = 1883;
const LIGHT_MODELS: [(&str, Model); 16] = [
    ("kitchen:fridge", Model::TradfriE14Color),
    ("kitchen:hallway", Model::TradfriE27),
    ("kitchen:hood_left", Model::TradfriCandle),
    ("kitchen:hood_right", Model::TradfriCandle),
    ("kitchen:ceiling", Model::HueGen4),
    ("large_bedroom:cabinet", Model::TradfriGU10),
    ("large_bedroom:ceiling", Model::TradfriE27),
    ("large_bedroom:desk", Model::TradfriE27),
    ("large_bedroom:wardrobe", Model::TradfriE27),
    ("large_bedroom:bed", Model::TradfriE14White),
    ("small_bedroom:ceiling", Model::HueGen4),
    ("small_bedroom:bureau", Model::HueGen4),
    ("small_bedroom:piano", Model::HueGen4),
    ("bathroom:ceiling", Model::HueGen4),
    ("hallway:ceiling", Model::TradfriE27),
    ("toilet:ceiling", Model::HueGen4),
];

#[derive(Debug, Clone)]
pub struct Controller {
    change_sender: mpsc::UnboundedSender<(String, LampProperty)>,
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

    pub fn set_on(&self, light_name: &str) {
        self.change_sender
            .send((light_name.to_string(), LampProperty::On(true)))
            .expect("Sender should never be dropped");
    }

    pub fn set_off(&self, light_name: &str) {
        self.change_sender
            .send((light_name.to_string(), LampProperty::On(false)))
            .expect("Sender should never be dropped");
    }

    /// Brightness from 0 to 1
    pub fn set_brightness(&self, light_name: &str, brightness: f64) {
        self.change_sender
            .send((
                light_name.to_string(),
                LampProperty::Brightness(brightness),
            ))
            .expect("Sender should never be dropped");
    }

    pub fn set_color_temp(&self, light_name: &str, kelvin: usize) {
        self.change_sender
            .send((
                light_name.to_string(),
                LampProperty::ColorTempK(kelvin),
            ))
            .expect("Sender should never be dropped");
    }

    pub fn set_color_xy(&self, light_name: &str, xy: (f64, f64)) {
        self.change_sender
            .send((light_name.to_string(), LampProperty::ColorXY(xy)))
            .expect("Sender should never be dropped");
    }
}
