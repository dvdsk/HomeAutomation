use serde_json::json;
use strum::EnumDiscriminants;

use crate::lights::{denormalize, kelvin_to_mired};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorTempStartup {
    Previous,
}

#[derive(Debug, EnumDiscriminants, Clone, Copy)]
#[strum_discriminants(derive(Hash))]
pub(crate) enum LampProperty {
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

pub(super) fn bri_is_close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1. / 250.
}

pub(super) fn temp_is_close(a: usize, b: usize) -> bool {
    a.abs_diff(b) < 50
}

pub(super) fn xy_is_close(a: (f64, f64), b: (f64, f64)) -> bool {
    let d_color_x = (a.0 - b.0).abs();
    let d_color_y = (a.1 - b.1).abs();
    d_color_x < 0.01 && d_color_y < 0.01
}
