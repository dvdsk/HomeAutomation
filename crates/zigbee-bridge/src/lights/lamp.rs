use std::collections::HashMap;

use serde_json::json;
use strum::EnumDiscriminants;
use tracing::instrument;

use crate::lights::{conversion::temp_to_xy, denormalize, kelvin_to_mired};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ColorTempStartup {
    Previous,
}

#[derive(Debug, EnumDiscriminants, Clone, Copy)]
#[strum_discriminants(derive(Hash))]
pub(super) enum LampProperty {
    Brightness(f64),
    ColorTempK(usize),
    ColorXY((f64, f64)),
    On(bool),
    ColorTempStartup(ColorTempStartup),
}

impl PartialEq for LampProperty {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LampProperty::Brightness(a), LampProperty::Brightness(b)) => {
                bri_is_close(*a, *b)
            }
            (LampProperty::ColorTempK(a), LampProperty::ColorTempK(b)) => {
                temp_is_close(*a, *b)
            }
            (LampProperty::ColorXY(a), LampProperty::ColorXY(b)) => {
                xy_is_close(*a, *b)
            }
            (LampProperty::On(a), LampProperty::On(b)) => a == b,
            (
                LampProperty::ColorTempStartup(a),
                LampProperty::ColorTempStartup(b),
            ) => a == b,
            (_, _) => false,
        }
    }
}

impl Eq for LampProperty {}

impl LampProperty {
    pub(crate) fn payload(&self) -> String {
        match *self {
            LampProperty::Brightness(bri) => {
                json!({ "brightness": denormalize(bri) })
            }
            LampProperty::ColorTempK(k) => {
                json!({ "color_temp": kelvin_to_mired(k) })
            }
            LampProperty::ColorXY((x, y)) => {
                json!({ "color": {"x": x, "y": y} })
            }
            LampProperty::On(lamp_on) if lamp_on => json!({"state": "ON"}),
            LampProperty::On(_) => json!({"state": "OFF"}),
            LampProperty::ColorTempStartup(ColorTempStartup::Previous) => {
                json!({"color_temp_startup": "previous"})
            }
        }
        .to_string()
    }
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

fn bri_is_close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1. / 250.
}

fn temp_is_close(a: usize, b: usize) -> bool {
    a.abs_diff(b) < 50
}

fn xy_is_close(a: (f64, f64), b: (f64, f64)) -> bool {
    let d_color_x = (a.0 - b.0).abs();
    let d_color_y = (a.1 - b.1).abs();
    d_color_x < 0.01 && d_color_y < 0.01
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
