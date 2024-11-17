use tracing::instrument;

use crate::lights::conversion::temp_to_xy;

pub(super) use model::Model;
pub(super) use property::{LampProperty, LampPropertyDiscriminants};

use property::{bri_is_close, temp_is_close, xy_is_close, ColorTempStartup};

mod model;
mod property;

#[derive(Debug, Clone)]
pub(super) struct LampState {
    pub(super) brightness: Option<f64>,
    pub(super) color_temp_k: Option<usize>,
    pub(super) color_xy: Option<(f64, f64)>,
    pub(super) on: Option<bool>,
    pub(super) color_temp_startup: ColorTempStartup,
}

impl Default for LampState {
    fn default() -> Self {
        Self {
            brightness: None,
            color_temp_k: None,
            color_xy: None,
            on: None,
            // Settings, will not be updated from MQTT state message
            // and will never trigger a publish
            color_temp_startup: ColorTempStartup::Previous,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub(super) struct Lamp {
    pub(super) model: Option<Model>,
    pub(super) state: LampState,
}

impl Lamp {
    pub(super) fn changes_relative_to(
        &self,
        other: &Self,
    ) -> Vec<LampProperty> {
        self.state
            .changes_relative_to(&other.state, self.model.as_ref())
    }

    pub(super) fn apply(self, change: Change) -> Self {
        Self {
            model: self.model,
            state: self.state.apply(change),
        }
    }

    pub(crate) fn property_list(&self) -> Vec<LampProperty> {
        self.state.property_list()
    }
}

impl LampState {
    #[instrument]
    pub(super) fn changes_relative_to(
        &self,
        other: &Self,
        model: Option<&Model>,
    ) -> Vec<LampProperty> {
        let mut res = Vec::new();
        if let Some(bri_self) = self.brightness {
            if other
                .brightness
                .is_none_or(|bri_other| !bri_is_close(bri_other, bri_self))
            {
                res.push(LampProperty::Brightness(bri_self));
            }
        }

        if let Some(temp_self) = self.color_temp_k {
            if other
                .color_temp_k
                .is_none_or(|temp_other| !temp_is_close(temp_other, temp_self))
            {
                res.push(LampProperty::ColorTempK(temp_self));
            }
        }

        if model.is_some_and(Model::is_color_lamp) {
            if let Some(xy_self) = self.color_xy {
                if other
                    .color_xy
                    .is_none_or(|xy_other| !xy_is_close(xy_other, xy_self))
                {
                    res.push(LampProperty::ColorXY(xy_self));
                }
            }
        }

        if let Some(on_self) = self.on {
            if other.on.is_none_or(|on_other| on_other != on_self) {
                res.push(LampProperty::On(on_self));
            }
        }

        if self.color_temp_startup != other.color_temp_startup {
            res.push(LampProperty::ColorTempStartup(self.color_temp_startup));
        }

        res
    }

    fn apply(self, change: Change) -> LampState {
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

    fn property_list(&self) -> Vec<LampProperty> {
        // we do not send color xy as the lamp might not support it
        // if it does then property_list is never called but an exact
        // diff between the current and need state is send

        let mut list = Vec::new();
        if let Some(val) = self.brightness {
            list.push(LampProperty::Brightness(val));
        }
        if let Some(val) = self.color_temp_k {
            list.push(LampProperty::ColorTempK(val));
        }
        if let Some(val) = self.on {
            list.push(LampProperty::On(val));
        }
        list.push(LampProperty::ColorTempStartup(self.color_temp_startup));
        list
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
                match (self.state.color_xy, other.state.color_xy) {
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
                match (self.state.color_temp_k, other.state.color_temp_k) {
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

        let bri_is_equal = match (self.state.brightness, other.state.brightness)
        {
            (Some(self_bri), Some(other_bri)) => {
                (self_bri - other_bri).abs() < 1. / 250.
            }
            _ => false,
        };

        self.state.on == other.state.on && bri_is_equal && color_is_equal
    }
}

impl Eq for Lamp {}

#[derive(Debug, Clone, Copy)]
pub(super) enum Change {
    On(bool),
    Brightness(f64),
    ColorTemp(usize),
    ColorXy((f64, f64)),
}
