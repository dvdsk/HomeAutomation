mod rooms;

use crate::system::System;
pub use protocol::SensorValue;
use tokio::sync::broadcast;
use tokio::task::JoinSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Sensor(SensorValue),
}

struct RestrictedSystem {
    allowed_lights: Vec<&'static str>,
    system: System,
}

impl RestrictedSystem {
    fn all_lamps_ct(&mut self, ct: u16, bri: u8) {
        for name in &self.allowed_lights {
            self.system.lights.set_ct(name, bri, ct).unwrap();
        }
    }
}

pub fn start(event_tx: broadcast::Sender<Event>, system: System) -> JoinSet<()> {
    let mut tasks = JoinSet::new();

    let restricted = RestrictedSystem {
        system,
        allowed_lights: vec![
            "large_bedroom:cabinet",
            "large_bedroom:ceiling",
            "large_bedroom:desk",
        ],
    };
    tasks.spawn(rooms::large_bedroom::run(
        event_tx.subscribe(),
        event_tx,
        restricted,
    ));

    tasks
}

