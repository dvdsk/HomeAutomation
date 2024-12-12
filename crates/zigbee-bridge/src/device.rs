use std::collections::HashMap;
use std::fmt::Debug;

use crate::lamp::{Lamp, LampProperty, LampPropertyDiscriminants};
use crate::radiator::{RadiatorProperty, RadiatorPropertyDiscriminants};
use crate::LIGHT_MODELS;

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum Property {
    Lamp(LampProperty),
    Radiator(RadiatorProperty),
}

impl Property {
    pub(crate) fn payload(&self) -> serde_json::Value {
        match self {
            Property::Lamp(lamp_prop) => lamp_prop.payload(),
            Property::Radiator(rad_prop) => rad_prop.payload(),
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum PropertyDiscriminants {
    Lamp(LampPropertyDiscriminants),
    Radiator(RadiatorPropertyDiscriminants),
}

impl Into<PropertyDiscriminants> for Property {
    fn into(self) -> PropertyDiscriminants {
        match self {
            Property::Lamp(lamp_prop) => {
                PropertyDiscriminants::Lamp(lamp_prop.into())
            }
            Property::Radiator(rad_prop) => {
                PropertyDiscriminants::Radiator(rad_prop.into())
            }
        }
    }
}

pub(crate) trait Device: Sync + Send + Debug {
    fn new(name: &str) -> Self
    where
        Self: Sized;

    fn clone_dyn(&self) -> Box<dyn Device>;

    fn apply(&mut self, change: Property);
    fn changes_relative_to(&self, other: &Box<dyn Device>) -> Vec<Property>;
    fn all_set_properties(&self) -> HashMap<PropertyDiscriminants, Property>;

    fn needs_merged_payloads(&self) -> bool;
    fn is_online(&self) -> bool;
}

impl Clone for Box<dyn Device> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

pub(crate) fn init_states() -> HashMap<String, Box<(dyn Device + 'static)>> {
    let mut states = HashMap::new();
    for (light_name, _) in LIGHT_MODELS {
        let device: Box<dyn Device> = Box::new(Lamp::new(light_name));
        states.insert(light_name.to_owned(), device);
    }
    states
}
