use serde_json::json;

use crate::lights::{conversion::temp_to_xy, denormalize, kelvin_to_mired};

#[derive(PartialEq, Eq, Default, Clone)]
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
    pub(super) color_sync: bool,
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
            color_sync: true,
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
        payloads.push(json!({ "color_sync": self.color_sync }));

        payloads.into_iter().map(|x| x.to_string()).collect()
    }
}

impl PartialEq for LampState {
    fn eq(&self, other: &Self) -> bool {
        let d_bright = match (self.brightness, other.brightness) {
            (Some(self_bri), Some(other_bri)) => (self_bri - other_bri).abs(),
            _ => 1.0,
        };
        let (d_color_x, d_color_y) = match (self.color_xy, other.color_xy) {
            (Some(self_xy), Some(other_xy)) => (
                (self_xy.0 - other_xy.0).abs(),
                (self_xy.1 - other_xy.1).abs(),
            ),
            // If either State has no xy set, xy is "different" (so we use temp)
            // so this needs to be over threshold
            _ => (1.0, 1.0),
        };
        let d_color_temp = match (self.color_temp_k, other.color_temp_k) {
            (Some(self_temp), Some(other_temp)) => {
                self_temp.abs_diff(other_temp)
            }
            _ => 5000,
        };

        let color_equal =
            (d_color_x < 0.01 && d_color_y < 0.01) || d_color_temp < 50;
        self.on == other.on && d_bright < 1. / 250. && color_equal
    }
}

impl Eq for LampState {}

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
    Other(String),
}

impl Model {
    fn is_color_lamp(&self) -> bool {
        use Model as M;
        #[allow(clippy::match_same_arms)] // clearer comment
        match self {
            M::TradfriE27 | M::TradfriE14 | M::HueGen4 => true,
            M::TradfriCandle => false,
            // We assume no so that things at least don't break
            M::TradfriOther(_) | M::Other(_) => false,
        }
    }
}
