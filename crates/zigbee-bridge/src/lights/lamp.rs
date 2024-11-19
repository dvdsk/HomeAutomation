pub(super) use model::Model;
pub(super) use property::{Property, PropertyDiscriminants};

use self::property::{bri_is_close, temp_is_close, xy_is_close};
use tracing::instrument;

use super::conversion::temp_to_xy;

mod model;
mod property;

#[derive(Default, Clone, Debug)]
pub(super) struct Lamp {
    model: Option<Model>,
    brightness: Option<f64>,
    color_temp_k: Option<usize>,
    color_xy: Option<(f64, f64)>,
    on: Option<bool>,
    color_temp_startup: property::ColorTempStartup,
}

impl Lamp {
    #[instrument]
    pub(super) fn changes_relative_to(&self, other: &Self) -> Vec<Property> {
        let mut res = Vec::new();
        if let Some(bri_self) = self.brightness {
            if other
                .brightness
                .is_none_or(|bri_other| !bri_is_close(bri_other, bri_self))
            {
                res.push(Property::Brightness(bri_self));
            }
        }

        if let Some(temp_self) = self.color_temp_k {
            if other
                .color_temp_k
                .is_none_or(|temp_other| !temp_is_close(temp_other, temp_self))
            {
                res.push(Property::ColorTempK(temp_self));
            }
        }

        if self.model.as_ref().is_some_and(Model::is_color_lamp) {
            if let Some(xy_self) = self.color_xy {
                if other
                    .color_xy
                    .is_none_or(|xy_other| !xy_is_close(xy_other, xy_self))
                {
                    res.push(Property::ColorXY(xy_self));
                }
            }
        }

        if let Some(on_self) = self.on {
            if other.on.is_none_or(|on_other| on_other != on_self) {
                res.push(Property::On(on_self));
            }
        }

        if self.color_temp_startup != other.color_temp_startup {
            res.push(Property::ColorTempStartup(self.color_temp_startup));
        }

        res
    }

    pub(super) fn apply(self, change: Property) -> Self {
        let mut new_state = self.clone();
        match change {
            Property::On(on) => new_state.on = Some(on),
            Property::Brightness(bri) => new_state.brightness = Some(bri),
            Property::ColorTempK(temp) => {
                new_state.color_temp_k = Some(temp);
                new_state.color_xy = Some(temp_to_xy(temp));
            }
            Property::ColorXY(xy) => new_state.color_xy = Some(xy),
            Property::ColorTempStartup(ct_startup) => {
                new_state.color_temp_startup = ct_startup
            }
        }
        new_state
    }

    pub(crate) fn property_list(&self) -> Vec<Property> {
        // we do not send color xy as the lamp might not support it
        // if it does then property_list is never called but an exact
        // diff between the current and need state is send

        let mut list = Vec::new();
        if let Some(val) = (&self).brightness {
            list.push(Property::Brightness(val));
        }
        if let Some(val) = (&self).color_temp_k {
            list.push(Property::ColorTempK(val));
        }
        if let Some(val) = (&self).on {
            list.push(Property::On(val));
        }
        list.push(Property::ColorTempStartup((&self).color_temp_startup));
        list
    }

    pub(crate) fn change_state(&mut self, property: Property) {
        match property {
            Property::Brightness(bri) => self.brightness = Some(bri),
            Property::ColorTempK(temp) => self.color_temp_k = Some(temp),
            Property::ColorXY(xy) => self.color_xy = Some(xy),
            Property::On(is_on) => self.on = Some(is_on),
            Property::ColorTempStartup(behavior) => {
                self.color_temp_startup = behavior;
            }
        }
    }

    pub(crate) fn set_model(&mut self, model: Model) {
        self.model = Some(model);
    }
}
