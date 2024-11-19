pub(super) use model::Model;
pub(super) use property::{Property, PropertyDiscriminants};

use self::property::{bri_is_close, temp_is_close, xy_is_close};
use tracing::instrument;

use super::conversion::temp_to_xy;

mod model;
mod property;

#[derive(Debug, Clone, Copy)]
pub(super) enum Change {
    On(bool),
    Brightness(f64),
    ColorTemp(usize),
    ColorXy((f64, f64)),
}

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

    pub(super) fn apply(self, change: Change) -> Self {
        let mut new_state = self.clone();
        match change {
            Change::On(on) => new_state.on = Some(on),
            Change::Brightness(bri) => new_state.brightness = Some(bri),
            Change::ColorTemp(temp) => {
                new_state.color_temp_k = Some(temp);
                new_state.color_xy = Some(temp_to_xy(temp));
            }
            Change::ColorXy(xy) => new_state.color_xy = Some(xy),
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

impl PartialEq for Lamp {
    fn eq(&self, other: &Self) -> bool {
        let color_is_equal = if let (Some(self_model), Some(other_model)) =
            (self.model.clone(), other.model.clone())
        {
            // We should only compare states for the same lamp
            assert_eq!(self_model, other_model);

            // We only ever set xy for color lamps,
            // so color temp doesn't say anything
            if self_model.is_color_lamp() {
                match (self.color_xy, other.color_xy) {
                    (Some(self_xy), Some(other_xy)) => {
                        let d_color_x = (self_xy.0 - other_xy.0).abs();
                        let d_color_y = (self_xy.1 - other_xy.1).abs();
                        d_color_x < 0.01 && d_color_y < 0.01
                    }
                    // If either State has no xy set, xy is unset -> different
                    _ => false,
                }
            // We only ever set temp, and xy doesn't exist
            } else {
                match (self.color_temp_k, other.color_temp_k) {
                    (Some(self_temp), Some(other_temp)) => {
                        self_temp.abs_diff(other_temp) < 50
                    }
                    _ => false,
                }
            }
        } else {
            // We don't know what model this is, thus we don't know how to compare
            // colors, so we assume unequal and hope that we know a model soon
            false
        };

        let bri_is_equal = match (self.brightness, other.brightness) {
            (Some(self_bri), Some(other_bri)) => {
                (self_bri - other_bri).abs() < 1. / 250.
            }
            _ => false,
        };

        self.on == other.on && bri_is_equal && color_is_equal
    }
}

impl Eq for Lamp {}
