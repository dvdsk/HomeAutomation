use serde_json::json;
use strum::EnumDiscriminants;
use tracing::error;

use super::Color;
use crate::lights::conversion::{denormalize, kelvin_to_mired};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorTempStartup {
    #[default]
    Previous,
}

#[derive(Debug, EnumDiscriminants, Clone, Copy)]
#[strum_discriminants(derive(Hash))]
pub(crate) enum Property {
    Online(bool),
    Brightness(f64),
    ColorTempK(usize),
    ColorXY((f64, f64)),
    On(bool),
    ColorTempStartup(ColorTempStartup),
}

impl PartialEq for Property {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Property::Brightness(a), Property::Brightness(b)) => {
                bri_is_close(*a, *b)
            }
            (Property::ColorTempK(a), Property::ColorTempK(b)) => {
                temp_is_close(*a, *b)
            }
            (Property::ColorXY(a), Property::ColorXY(b)) => xy_is_close(*a, *b),
            (Property::On(a), Property::On(b)) => a == b,
            (Property::ColorTempStartup(a), Property::ColorTempStartup(b)) => {
                a == b
            }
            // we never need to update this, thus we always consider it the same
            (Property::Online(_), Property::Online(_)) => true,
            (_, _) => false,
        }
    }
}

impl Eq for Property {}

impl Property {
    pub(crate) fn payload(&self) -> serde_json::Value {
        match *self {
            Property::Brightness(bri) => {
                json!({ "brightness": denormalize(bri) })
            }
            Property::ColorTempK(k) => {
                json!({ "color_temp": kelvin_to_mired(k) })
            }
            Property::ColorXY((x, y)) => {
                json!({ "color": {"x": x, "y": y} })
            }
            Property::On(lamp_on) if lamp_on => json!({"state": "ON"}),
            Property::On(_) => json!({"state": "OFF"}),
            Property::ColorTempStartup(ColorTempStartup::Previous) => {
                json!({"color_temp_startup": "previous"})
            }
            // read-only, shouldn't be called, safe default
            Property::Online(_) => {
                error!("Tried to convert Online to payload");
                json!({"state": ""})
            }
        }
    }
}

pub(super) fn bri_is_close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1. / 250.
}

pub(super) fn color_is_close(a: Color, b: Color) -> Result<bool, String> {
    match (a, b) {
        (Color::XY(a), Color::XY(b)) => Ok(xy_is_close(a, b)),
        (Color::TempK(a), Color::TempK(b)) => Ok(temp_is_close(a, b)),
        (_, _) => Err("Comparing XY and Temp".to_owned()),
    }
}

fn temp_is_close(a: usize, b: usize) -> bool {
    a.abs_diff(b) < 50
}

fn xy_is_close(a: (f64, f64), b: (f64, f64)) -> bool {
    let d_color_x = (a.0 - b.0).abs();
    let d_color_y = (a.1 - b.1).abs();
    d_color_x < 0.01 && d_color_y < 0.01
}
