#![allow(clippy::missing_panics_doc)]
use std::net::IpAddr;

use tokio::sync::mpsc;
use tracing::trace;

pub(crate) use lamp::Model;

use self::{device::Property, lamp::LampProperty, radiator::RadiatorProperty};

mod cached_bridge;
mod conversion;
mod device;
pub(crate) mod lamp;
mod parse;
mod radiator;

const MQTT_PORT: u16 = 1883;
const LIGHT_MODELS: [(&str, Model); 16] = [
    ("kitchen:fridge", Model::TradfriE14Color),
    ("kitchen:hallway", Model::TradfriE27),
    ("kitchen:hood_left", Model::TradfriCandle),
    ("kitchen:hood_right", Model::TradfriCandle),
    ("kitchen:ceiling", Model::TradfriE27),
    ("large_bedroom:cabinet", Model::TradfriGU10),
    ("large_bedroom:ceiling", Model::TradfriE27),
    ("large_bedroom:desk", Model::TradfriE27),
    ("large_bedroom:wardrobe", Model::TradfriE27),
    ("large_bedroom:bed", Model::TradfriE14White),
    ("small_bedroom:ceiling", Model::HueGen1),
    ("small_bedroom:bureau", Model::HueGen1),
    ("small_bedroom:piano", Model::TradfriE27),
    ("bathroom:ceiling", Model::HueGen1),
    ("hallway:ceiling", Model::TradfriE27),
    ("toilet:ceiling", Model::HueGen2),
];
const RADIATOR_NAMES: [&str; 2] =
    ["small_bedroom:radiator", "large_bedroom:radiator"];

fn light_names() -> Vec<&'static str> {
    LIGHT_MODELS.into_iter().map(|(k, _)| k).collect()
}

#[derive(Debug, Clone)]
pub struct Controller {
    change_sender: mpsc::UnboundedSender<(String, Property)>,
}

impl Controller {
    #[must_use]
    pub fn start_bridge(mqtt_ip: IpAddr, name: &str) -> Self {
        Self::start_bridge_with_reading_callback(mqtt_ip, name, |_| ())
    }
    #[must_use]
    pub fn start_bridge_with_reading_callback(
        mqtt_ip: IpAddr,
        name: &str,
        reading_callback: impl Fn(protocol::Reading) + Send + 'static,
    ) -> Self {
        let (change_sender, change_receiver) = mpsc::unbounded_channel();

        let run_bridge = cached_bridge::run(
            mqtt_ip,
            change_receiver,
            name.to_string(),
            reading_callback,
        );
        trace!("Spawning zigbee bridge task");
        tokio::task::spawn(run_bridge);

        Self { change_sender }
    }

    pub fn set_on(&self, light_name: &str) {
        self.send_to_light(light_name, LampProperty::On(true));
    }

    pub fn set_off(&self, light_name: &str) {
        self.send_to_light(light_name, LampProperty::On(false));
    }

    /// Brightness from 0 to 1
    pub fn set_brightness(&self, light_name: &str, brightness: f64) {
        self.send_to_light(light_name, LampProperty::Brightness(brightness));
    }

    pub fn set_color_temp(&self, light_name: &str, kelvin: usize) {
        self.send_to_light(light_name, LampProperty::ColorTempK(kelvin));
    }

    pub fn set_color_xy(&self, light_name: &str, xy: (f64, f64)) {
        self.send_to_light(light_name, LampProperty::ColorXY(xy));
    }

    pub fn set_radiator_setpoint(&self, radiator_name: &str, setpoint: f64) {
        self.change_sender
            .send((
                radiator_name.to_owned(),
                RadiatorProperty::Setpoint(setpoint).into(),
            ))
            .expect("Sender should never be dropped");
    }

    pub fn set_radiator_reference(&self, radiator_name: &str, reference: f64) {
        self.change_sender
            .send((
                radiator_name.to_owned(),
                RadiatorProperty::Reference(reference).into(),
            ))
            .expect("Sender should never be dropped");
    }

    fn send_to_light(&self, light_name: &str, lamp_property: LampProperty) {
        self.change_sender
            .send((light_name.to_string(), lamp_property.into()))
            .expect("Sender should never be dropped");
    }
}
