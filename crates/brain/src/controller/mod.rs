mod rooms;

use std::future::Future;

use crate::system::System;
use color_eyre::eyre::Context;
pub use protocol::Reading;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tokio::task::{JoinSet, LocalSet};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Event {
    Sensor(Reading),
    WakeupLB,
    WakeupSB,
    WakeupKitchen,
    StateChangeSB(rooms::small_bedroom::State),
}

#[derive(Clone)]
pub(crate) struct RestrictedSystem {
    allowed_lights: Vec<&'static str>,
    allowed_radiators: Vec<&'static str>,
    system: System,
}

impl RestrictedSystem {
    async fn one_lamp_ct(
        &mut self,
        name: &'static str,
        kelvin: usize,
        bri: f64,
    ) {
        if self.allowed_lights.contains(&name) {
            self.system.zigbee.set_color_temp(name, kelvin);
            self.system.zigbee.set_brightness(name, bri);
        }
    }

    async fn one_lamp_on(&mut self, name: &'static str) {
        if self.allowed_lights.contains(&name) {
            self.system.zigbee.set_on(name);
        }
    }

    async fn one_lamp_off(&mut self, name: &'static str) {
        if self.allowed_lights.contains(&name) {
            self.system.zigbee.set_off(name);
        }
    }

    async fn all_lamps_ct(&mut self, kelvin: usize, bri: f64) {
        for name in &self.allowed_lights {
            self.system.zigbee.set_color_temp(name, kelvin);
            self.system.zigbee.set_brightness(name, bri);
        }
    }

    async fn all_lamps_off(&mut self) {
        for name in &self.allowed_lights {
            self.system.zigbee.set_off(name);
        }
    }

    async fn all_lamps_on(&mut self) {
        for name in &self.allowed_lights {
            self.system.zigbee.set_on(name);
        }
    }

    #[allow(unused)]
    async fn all_lamps_but_one_off(&mut self, leave_this_on: &str) {
        for name in &self.allowed_lights {
            if *name != leave_this_on {
                self.system.zigbee.set_off(name);
            }
        }
    }

    #[allow(unused)]
    async fn all_lamps_but_one_on(&mut self, leave_this_off: &str) {
        for name in &self.allowed_lights {
            if *name != leave_this_off {
                self.system.zigbee.set_on(name);
            }
        }
    }

    async fn set_radiators_setpoint(&mut self, temperature: f64) {
        for name in &self.allowed_radiators {
            self.system.zigbee.set_radiator_setpoint(name, temperature);
        }
    }
}

#[derive(Default)]
pub struct ControllerTasks {
    tasks: JoinSet<Result<(), color_eyre::Report>>,
    local_set: LocalSet,
}

impl ControllerTasks {
    fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = Result<(), color_eyre::Report>> + 'static,
    {
        self.tasks.spawn_local_on(future, &self.local_set);
    }

    pub async fn report_failed(&mut self) {
        while let Some(failure) = self.tasks.join_next().await {
            match failure {
                Ok(Ok(())) => {
                    tracing::error!("Task returned, tasks should never return!")
                }
                Ok(Err(e)) => tracing::error!("Task returned an error: {e}"),
                Err(e) => tracing::error!("Task could not join: {e}"),
            }
        }
    }
}

pub fn start(
    subscribed: [broadcast::Receiver<Event>; 4],
    sender: broadcast::Sender<Event>,
    system: System,
    db: sled::Db,
) -> Result<ControllerTasks, color_eyre::Report> {
    tracing::info!("starting");
    let mut tasks = ControllerTasks::default();
    let [rx1, rx2, rx3, rx4] = subscribed;

    let restricted = RestrictedSystem {
        system: system.clone(),
        allowed_lights: vec![
            "large_bedroom:cabinet",
            "large_bedroom:ceiling",
            "large_bedroom:desk",
            "large_bedroom:wardrobe",
            "large_bedroom:bed",
        ],
        allowed_radiators: vec!["large_bedroom:radiator"],
    };
    let ds = db
        .open_tree("large_bedroom")
        .wrap_err("Opening db subtree for large_bedroom")?;
    tasks.spawn(rooms::large_bedroom::run(
        rx1,
        sender.clone(),
        restricted,
        ds,
    ));

    let restricted = RestrictedSystem {
        system: system.clone(),
        allowed_lights: vec![
            "small_bedroom:ceiling",
            "small_bedroom:bureau",
            "small_bedroom:piano",
        ],
        allowed_radiators: vec!["small_bedroom:radiator"],
    };
    let ds = db
        .open_tree("small_bedroom")
        .wrap_err("Opening db subtree for small_bedroom")?;
    tasks.spawn(rooms::small_bedroom::run(
        rx2,
        sender.clone(),
        restricted,
        ds,
    ));

    let restricted = RestrictedSystem {
        system: system.clone(),
        allowed_lights: vec![
            "kitchen:ceiling",
            "kitchen:hood_left",
            "kitchen:hood_right",
            "kitchen:fridge",
            "kitchen:hallway",
        ],
        allowed_radiators: vec![],
    };
    let ds = db
        .open_tree("kitchen")
        .wrap_err("Opening db subtree for kitchen")?;
    tasks.spawn(rooms::kitchen::run(rx3, sender.clone(), restricted, ds));

    let restricted = RestrictedSystem {
        system: system.clone(),
        allowed_lights: vec![
            "hallway:ceiling",
            "bathroom:ceiling",
            "toilet:ceiling",
        ],
        allowed_radiators: vec![],
    };
    let ds = db
        .open_tree("entrance")
        .wrap_err("Opening db subtree for entrance")?;
    tasks.spawn(rooms::entrance::run(rx4, sender, restricted, ds));

    Ok(tasks)
}
