use std::collections::HashMap;

use strum::IntoEnumIterator;
use tracing::{error, instrument};

use super::conversion::temp_to_xy;
use crate::device::{Device, Property, PropertyDiscriminants};
pub(crate) use model::Model;
pub(crate) use property::{LampProperty, LampPropertyDiscriminants};

mod model;
mod property;
mod color_correction;

#[derive(Clone, Copy, Debug)]
enum Color {
    TempK(usize),
    XY((f64, f64)),
}

impl Color {
    fn xy_from_temp(temp: usize, model: &Model) -> Color {
        Color::XY(temp_to_xy(temp, model.color_deviation(temp)))
    }
}

// TODO: some way to enforce read-only (thus known-updatable-only) fields?
#[derive(Clone, Debug)]
pub(crate) struct Lamp {
    model: Model,
    is_online: bool,
    brightness: Option<f64>,
    color: Option<Color>,
    is_on: Option<bool>,
    color_temp_startup: property::ColorTempStartup,
}

impl Device for Lamp {
    fn new(name: &str) -> Self {
        Self {
            model: Model::from_light(name),
            // we assume the lamp is online so that init messages get sent
            is_online: true,
            brightness: None,
            color: None,
            is_on: None,
            color_temp_startup: property::ColorTempStartup::default(),
        }
    }

    fn clone_dyn(&self) -> Box<dyn Device> {
        Box::new(self.clone())
    }

    fn needs_merged_payloads(&self) -> bool {
        self.model.is_hue()
    }

    fn is_online(&self) -> bool {
        self.is_online
    }

    #[instrument]
    fn changes_relative_to(&self, other: &dyn Device) -> Vec<Property> {
        let mut res = Vec::new();

        let self_properties = self.all_set_properties();
        let other_properties = other.all_set_properties();

        for property in LampProperty::iter() {
            let self_prop = self_properties.get(&property.into());
            let other_prop = other_properties.get(&property.into());

            // Ignore model (not a prop) and online, because they are read-only
            if let Some(Property::Lamp(LampProperty::Online(_))) = self_prop {
                continue;
            }

            if let Some(self_prop) = self_prop {
                if other_prop.is_none_or(|other_prop| self_prop != other_prop) {
                    res.push(*self_prop);
                }
            }
        }

        res
    }

    fn apply(&mut self, change: Property) {
        let Property::Lamp(change) = change else {
            error!("Trying to apply non-lamp change {change:?} to lamp!");
            return;
        };

        match change {
            LampProperty::On(is_on) => self.is_on = Some(is_on),
            LampProperty::Brightness(bri) => self.brightness = Some(bri),
            LampProperty::ColorTempK(temp) => {
                // if we know the model, we know how to apply temp
                if self.model.supports_xy() {
                    self.color = Some(Color::xy_from_temp(temp, &self.model));
                } else {
                    let range = self.model.temp_k_range();
                    let temp = temp.clamp(range.start, range.end);
                    self.color = Some(Color::TempK(temp))
                }
            }
            LampProperty::ColorXY(xy) => {
                // don't apply xy to unknown or non-color lamp
                if self.model.supports_xy() {
                    self.color = Some(Color::XY(xy))
                }
            }
            LampProperty::ColorTempStartup(behaviour) => {
                self.color_temp_startup = behaviour
            }
            LampProperty::Online(is_online) => self.is_online = is_online,
        }
    }

    fn all_set_properties(&self) -> HashMap<PropertyDiscriminants, Property> {
        let mut properties: HashMap<PropertyDiscriminants, Property> =
            HashMap::new();

        let mut insert_prop =
            |prop: LampProperty| properties.insert(prop.into(), prop.into());

        // Ignore model and is_online, because they are read-only

        if let Some(val) = self.brightness {
            insert_prop(LampProperty::Brightness(val));
        }
        if let Some(val) = self.color {
            match val {
                Color::XY(xy) => {
                    insert_prop(LampProperty::ColorXY(xy));
                }
                Color::TempK(temp) => {
                    insert_prop(LampProperty::ColorTempK(temp));
                }
            }
        }
        if let Some(val) = self.is_on {
            insert_prop(LampProperty::On(val));
        }
        insert_prop(LampProperty::ColorTempStartup(self.color_temp_startup));
        properties
    }
}
