use std::collections::HashMap;
use std::fmt::Debug;

use crate::{lights::lamp::{property::{LampProperty, LampPropertyDiscriminants}, Lamp}, LIGHT_MODELS};

pub(crate) trait Device: Sync + Send + Debug {
    fn new(name: &str) -> Self
    where
        Self: Sized;

    fn clone_dyn(&self) -> Box<dyn Device>;

    fn apply(&mut self, change: LampProperty);
    fn changes_relative_to(&self, other: &Box<dyn Device>) -> Vec<LampProperty>;
    fn all_set_properties(
        &self,
    ) -> HashMap<LampPropertyDiscriminants, LampProperty>;

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
