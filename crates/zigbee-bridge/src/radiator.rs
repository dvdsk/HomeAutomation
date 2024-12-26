use std::{collections::HashMap, str::FromStr};

use serde_json::json;
use strum::{EnumDiscriminants, EnumIter, IntoEnumIterator};
use tracing::{error, instrument};

use crate::{
    conversion::{round_to_half, times_100_int},
    device::{Device, Property, PropertyDiscriminants},
};

#[derive(Debug, Clone)]
pub(crate) struct Radiator {
    is_online: bool,
    setpoint: Option<f64>,
    reference: Option<f64>,
    set_by_method: Option<SetMethod>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum SetMethod {
    Manual,
    Schedule,
    #[default]
    External,
}

impl FromStr for SetMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "manual" => Ok(SetMethod::Manual),
            "schedule" => Ok(SetMethod::Schedule),
            "externally" => Ok(SetMethod::External),
            invalid => {
                Err(format!("Unknown set method encountered: {invalid}"))
            }
        }
    }
}

impl Device for Radiator {
    fn new(_name: &str) -> Self
    where
        Self: Sized,
    {
        Self {
            is_online: true,
            setpoint: None,
            reference: None,
            set_by_method: None,
        }
    }

    fn clone_dyn(&self) -> Box<dyn Device> {
        Box::new(self.clone())
    }

    #[instrument]
    fn apply(&mut self, change: Property) {
        let Property::Radiator(change) = change else {
            error!(
                "Trying to apply non-radiator change {change:?} to radiator!"
            );
            return;
        };

        match change {
            RadiatorProperty::Online(is_online) => self.is_online = is_online,
            RadiatorProperty::Setpoint(setpoint) => {
                self.setpoint = Some(setpoint)
            }
            RadiatorProperty::Reference(reference) => {
                self.reference = Some(reference)
            }
            // Ignore radiator reporting reference expired, so we don't start
            // sending an old value
            RadiatorProperty::NoReference => (),
            RadiatorProperty::SetByMethod(set_method) => {
                self.set_by_method = Some(set_method)
            }
        }
    }

    fn changes_relative_to(&self, other: &dyn Device) -> Vec<Property> {
        let mut res = Vec::new();

        let self_properties = self.all_set_properties();
        let other_properties = other.all_set_properties();

        for property in RadiatorProperty::iter() {
            let self_prop = self_properties.get(&property.into());
            let other_prop = other_properties.get(&property.into());

            // Ignore set_by_method and online, because they are read-only
            if let Some(Property::Radiator(
                RadiatorProperty::Online(_) | RadiatorProperty::SetByMethod(_),
            )) = self_prop
            {
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

    fn all_set_properties(
        &self,
    ) -> std::collections::HashMap<PropertyDiscriminants, Property> {
        let mut properties: HashMap<PropertyDiscriminants, Property> =
            HashMap::new();

        let mut insert_prop = |prop: RadiatorProperty| {
            properties.insert(prop.into(), prop.into())
        };

        // Ignore set_by_method and online, because they are read-only

        if let Some(val) = self.setpoint {
            insert_prop(RadiatorProperty::Setpoint(val));
        }
        if let Some(val) = self.reference {
            insert_prop(RadiatorProperty::Reference(val));
        }

        properties
    }

    fn needs_merged_payloads(&self) -> bool {
        false
    }

    fn is_online(&self) -> bool {
        self.is_online
    }
}

#[derive(Debug, EnumDiscriminants, EnumIter, Clone, Copy)]
#[strum_discriminants(derive(Hash))]
pub(crate) enum RadiatorProperty {
    Online(bool),
    Setpoint(f64),
    Reference(f64),
    NoReference,
    SetByMethod(SetMethod),
}

impl PartialEq for RadiatorProperty {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (RadiatorProperty::Setpoint(a), RadiatorProperty::Setpoint(b)) => {
                round_to_half(a) == round_to_half(b)
            }
            (
                RadiatorProperty::Reference(a),
                RadiatorProperty::Reference(b),
            ) => times_100_int(a) == times_100_int(b),
            // we never need to update these, so we always consider them the same
            (RadiatorProperty::Online(_), RadiatorProperty::Online(_)) => true,
            (
                RadiatorProperty::SetByMethod(_),
                RadiatorProperty::SetByMethod(_),
            ) => true,
            (_, _) => false,
        }
    }
}

impl RadiatorProperty {
    pub(crate) fn payload(&self) -> serde_json::Value {
        match *self {
            RadiatorProperty::Setpoint(setpoint) => {
                json!({"occupied_heating_setpoint": round_to_half(setpoint) })
            }
            RadiatorProperty::Reference(reference) => {
                json!({"external_measured_room_sensor": times_100_int(reference) })
            }
            // read-only, shouldn't be called, safe default
            RadiatorProperty::NoReference => {
                error!("Tried to convert NoReference to payload");
                json!({"state": ""})
            }
            // read-only, shouldn't be called, safe default
            RadiatorProperty::Online(_) => {
                error!("Tried to convert Online to payload");
                json!({"state": ""})
            }
            // read-only, shouldn't be called, safe default
            RadiatorProperty::SetByMethod(_) => {
                error!("Tried to convert SetByMethod to payload");
                json!({"state": ""})
            }
        }
    }
}

impl From<RadiatorProperty> for Property {
    fn from(value: RadiatorProperty) -> Self {
        Self::Radiator(value)
    }
}

impl From<RadiatorProperty> for PropertyDiscriminants {
    fn from(value: RadiatorProperty) -> Self {
        Self::Radiator(value.into())
    }
}

impl TryFrom<Property> for RadiatorProperty {
    type Error = String;

    fn try_from(value: Property) -> Result<Self, Self::Error> {
        match value {
            Property::Radiator(lamp_prop) => Ok(lamp_prop),
            _ => Err(
                "Tried to interpret non-radiator property as radiator property"
                    .to_owned(),
            ),
        }
    }
}
