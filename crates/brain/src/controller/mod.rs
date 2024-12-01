mod rooms;
pub(crate) use rooms::large_bedroom;

use crate::system::System;
pub use protocol::Reading;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::task::JoinSet;
use zigbee_bridge::lights::{mired_to_kelvin, normalize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Sensor(Reading),
    WakeUp,
}

#[derive(Clone)]
pub(crate) struct RestrictedSystem {
    allowed_lights: Vec<&'static str>,
    allowed_lights_new: Vec<&'static str>,
    system: System,
}

impl RestrictedSystem {
    async fn one_lamp_ct(&mut self, name: &'static str, ct: u16, bri: u8) {
        if self.allowed_lights.contains(&name) {
            self.system.lights.set_ct(name, bri, ct).await.unwrap();
        }

        if self.allowed_lights_new.contains(&name) {
            self.system
                .lights_new
                .set_color_temp(name, mired_to_kelvin(ct.into()));
            self.system.lights_new.set_brightness(name, normalize(bri));
        }
    }

    async fn one_lamp_on(&mut self, name: &'static str) {
        if self.allowed_lights.contains(&name) {
            self.system.lights.single_on(name).await.unwrap();
        }

        if self.allowed_lights_new.contains(&name) {
            self.system.lights_new.set_on(name);
        }
    }

    async fn one_lamp_off(&mut self, name: &'static str) {
        if self.allowed_lights.contains(&name) {
            self.system.lights.single_off(name).await.unwrap();
        }

        if self.allowed_lights_new.contains(&name) {
            self.system.lights_new.set_off(name);
        }
    }

    async fn all_lamps_ct(&mut self, ct: u16, bri: u8) {
        for name in &self.allowed_lights {
            self.system.lights.set_ct(name, bri, ct).await.unwrap();
        }

        for name in &self.allowed_lights_new {
            self.system
                .lights_new
                .set_color_temp(name, mired_to_kelvin(ct.into()));
            self.system.lights_new.set_brightness(name, normalize(bri));
        }
    }

    async fn all_lamps_off(&mut self) {
        for name in &self.allowed_lights {
            self.system.lights.single_off(name).await.unwrap();
        }

        for name in &self.allowed_lights_new {
            self.system.lights_new.set_off(name);
        }
    }

    async fn all_lamps_on(&mut self) {
        for name in &self.allowed_lights {
            self.system.lights.single_on(name).await.unwrap();
        }

        for name in &self.allowed_lights_new {
            self.system.lights_new.set_on(name);
        }
    }
}

pub fn start(
    subscribed: [broadcast::Receiver<Event>; 3],
    sender: broadcast::Sender<Event>,
    system: System,
) -> JoinSet<()> {
    tracing::info!("starting");
    let mut tasks = JoinSet::new();
    let [rx1, rx2, rx3] = subscribed;

    let restricted = RestrictedSystem {
        system: system.clone(),
        allowed_lights: vec![
            "large_bedroom:cabinet",
            "large_bedroom:ceiling",
            "large_bedroom:desk",
            "large_bedroom:wardrobe",
            "large_bedroom:bed",
        ],
        allowed_lights_new: vec![],
    };
    tasks.spawn(rooms::large_bedroom::run(rx1, sender.clone(), restricted));

    let restricted = RestrictedSystem {
        system: system.clone(),
        allowed_lights: vec![
            "small_bedroom:ceiling",
            "small_bedroom:bureau",
            "small_bedroom:piano",
        ],
        allowed_lights_new: vec![],
    };
    tasks.spawn(rooms::small_bedroom::run(rx2, sender.clone(), restricted));

    let restricted = RestrictedSystem {
        system: system.clone(),
        allowed_lights: vec![],
        allowed_lights_new: vec![
            "kitchen:ceiling",
            "kitchen:hood_left",
            "kitchen:hood_right",
            "kitchen:fridge",
            "kitchen:hallway",
        ],
    };
    tasks.spawn(rooms::kitchen::run(rx3, sender, restricted));

    tasks
}
