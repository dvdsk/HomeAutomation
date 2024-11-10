use std::collections::HashMap;

use serde_json::json;

use crate::lights::{conversion::temp_to_xy, denormalize, kelvin_to_mired};

#[derive(Default, Clone, Debug)]
pub(super) struct Lamp {
    pub(super) model: Option<Model>,
    pub(super) state: LampState,
}

#[derive(Debug, Clone)]
pub(super) struct LampState {
    pub(super) brightness: Option<f64>,
    pub(super) color_temp_k: Option<usize>,
    pub(super) color_xy: Option<(f64, f64)>,
    pub(super) on: Option<bool>,
    pub(super) color_temp_startup: String,
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
            color_temp_startup: String::from("previous"),
        }
    }
}

impl Lamp {
    pub(super) fn store_model(&self, model: Model) -> Self {
        Self {
            model: Some(model),
            state: self.state.clone(),
        }
    }

    pub(super) fn store_state(&self, state: LampState) -> Self {
        Self {
            model: self.model.clone(),
            state,
        }
    }

    pub(super) fn apply(&self, change: Change) -> Self {
        Self {
            model: self.model.clone(),
            state: self.state.apply(change),
        }
    }

    pub(super) fn to_payloads(&self) -> Vec<String> {
        self.state.to_payloads(&self.model)
    }
}

impl LampState {
    fn apply(&self, change: Change) -> LampState {
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

    fn to_payloads(&self, model: &Option<Model>) -> Vec<String> {
        let mut payloads = vec![];
        match self.on {
            Some(true) => payloads.push(json!({ "state": "ON" })),
            Some(false) => payloads.push(json!({ "state": "OFF" })),
            None => (),
        };

        if let Some(bri) = self.brightness {
            payloads.push(json!({ "brightness": denormalize(bri) }));
        }

        if let Some(model) = model {
            if model.is_color_lamp() {
                if let Some(color_xy) = self.color_xy {
                    payloads.push(
                        json!({ "color": {"x": color_xy.0, "y": color_xy.1} }),
                    );
                }
            } else if let Some(color_temp) = self.color_temp_k {
                payloads
                    .push(json!({ "color_temp": kelvin_to_mired(color_temp) }));
            }
        }

        payloads.push(json!({ "color_temp_startup": self.color_temp_startup }));

        payloads.into_iter().map(|x| x.to_string()).collect()
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Model {
    TradfriCandle,
    TradfriE27,
    TradfriE14,
    HueGen4,
    TradfriOther(String),
    HueOther(String),
    Other(String),
}

impl Model {
    fn is_color_lamp(&self) -> bool {
        #[allow(clippy::match_same_arms)] // clearer comment
        match self {
            Model::TradfriE27 | Model::TradfriE14 | Model::HueGen4 => true,
            Model::TradfriCandle => false,
            // We assume no so that things at least don't break
            Model::TradfriOther(_) | Model::HueOther(_) | Model::Other(_) => {
                false
            }
        }
    }

    fn color_deviation(&self, color_temp: usize) -> (usize, f64) {
        match self {
            Model::TradfriCandle => todo!(),
            Model::TradfriE27 => todo!(),
            Model::TradfriE14 => todo!(),
            Model::HueGen4 => todo!(),
            Model::TradfriOther(_) => todo!(),
            Model::HueOther(_) => todo!(),
            Model::Other(_) => todo!(),
        }
    }

    // Value for actual color temp (after temp correction)
    fn blackbody_table(&self) -> HashMap<usize, f64> {
        match self {
            // 105.455.00 data
            Model::TradfriCandle => vec![
                (2170, 0.0028),
                (2391, 0.0018),
                (2683, 0.0006),
                (2990, 0.0006),
                (3193, 0.0012),
                (3590, 0.003),
                (3791, 0.0036),
                (3951, 0.0046),
            ],
            // 604.391.68 data
            Model::TradfriE27 => vec![
                (1964, -0.003),
                (2090, 0.0014),
                (2668, 0.0015),
                (3046, 0.001),
                (3499, 0.0015),
                (3605, -0.0011),
                (4098, -0.0017),
                (4110, -0.0009),
                (4250, 0.0007),
            ],
            // 204.391.94 data
            Model::TradfriE14 => vec![
                (1841, -0.0032),
                (2055, -0.0034),
                (2120, 0.0012),
                (2738, -0.0005),
                (3070, 0.0004),
                (3467, 0.0045),
                (3555, 0.00),
                (3981, 0.0009),
                (4113, 0.0037),
            ],
            // A19 Color 9.5W data
            Model::HueGen4 | Model::HueOther(_) => vec![
                (1998, 0.0005),
                (2197, 0.00000),
                (2519, -0.0004),
                (2695, -0.0007),
                (2849, -0.0011),
                (3358, -0.0012),
                (3864, -0.012),
                (4010, -0.0009),
                (5455, -0.0005),
                (6495, 0.0014),
            ],
            Model::TradfriOther(_) => todo!(),
            Model::Other(_) => todo!(),
        }
        .into_iter()
        .collect()
    }

    // Value to add to requested color temp
    fn temp_table(&self) -> HashMap<usize, isize> {
        match self {
            Model::TradfriCandle => {
                vec![(2200, 30), (2700, 20), (4000, 50)]
            }
            Model::TradfriE27 => vec![(2200, 110), (2700, 30), (4000, -110)],
            Model::TradfriE14 => vec![(2200, 70), (2700, -40), (4000, 20)],
            Model::HueGen4 | Model::HueOther(_) => {
                vec![(2200, 2), (2700, 5), (4000, -10), (5500, 45), (6500, 5)]
            }
            Model::TradfriOther(_) => todo!(),
            Model::Other(_) => todo!(),
        }
        .into_iter()
        .collect()
    }
}
