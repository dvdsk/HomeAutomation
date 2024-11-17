pub(super) use model::Model;
pub(super) use property::{LampProperty, LampPropertyDiscriminants};

use state::LampState;

mod model;
mod property;
mod state;

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
    state: LampState,
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

    pub(crate) fn change_state(&mut self, property: LampProperty) {
        let state = &mut self.state;
        match property {
            LampProperty::Brightness(bri) => state.brightness = Some(bri),
            LampProperty::ColorTempK(temp) => state.color_temp_k = Some(temp),
            LampProperty::ColorXY(xy) => state.color_xy = Some(xy),
            LampProperty::On(is_on) => state.on = Some(is_on),
            LampProperty::ColorTempStartup(behavior) => {
                state.color_temp_startup = behavior;
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
