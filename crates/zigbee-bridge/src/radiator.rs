use strum::EnumDiscriminants;

use crate::device::{Property, PropertyDiscriminants};

#[derive(PartialEq, Eq, EnumDiscriminants, Debug, Clone, Copy)]
#[strum_discriminants(derive(Hash))]
pub(crate) enum RadiatorProperty {
    SetPoint,
    Online(bool),
}

impl RadiatorProperty {
    pub(crate) fn payload(&self) -> serde_json::Value {
        todo!()
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
