use std::io;

use serde_json::{json, Value};

use crate::lights::{
    conversion::{normalize, temp_to_xy},
    denormalize, kelvin_to_mired, mired_to_kelvin,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct Lamp {
    pub(crate) brightness: Option<f64>,
    pub(crate) color_temp_k: Option<usize>,
    pub(crate) color_xy: Option<(f64, f64)>,
    pub(crate) on: Option<bool>,
}

impl PartialEq for Lamp {
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
            // If either Lamp has no xy set, xy is "different" (so we use temp)
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

impl Eq for Lamp {}

impl TryInto<Lamp> for &[u8] {
    type Error = io::Error;

    fn try_into(self) -> Result<Lamp, Self::Error> {
        fn get_key<'a>(
            map: &'a serde_json::Map<String, Value>,
            key: &str,
        ) -> Result<&'a Value, io::Error> {
            let key_err = io::Error::new(
                io::ErrorKind::InvalidData,
                "Missing key from map: {key}",
            );
            map.get(key).ok_or(key_err)
        }

        fn invalid_err(value_type: &str) -> io::Error {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Could not parse json into {value_type}"),
            )
        }

        let json: Value = serde_json::from_slice(self)?;
        let map = json.as_object().ok_or(invalid_err("Object"))?;

        let color_temp_mired = match get_key(map, "color_temp") {
            Ok(temp) => {
                let color_temp: usize = temp
                    .as_number()
                    .ok_or(invalid_err("Number"))?
                    .as_u64()
                    .ok_or(invalid_err("u64"))?
                    .try_into()
                    .expect("usize should be u64");
                Some(color_temp)
            }
            Err(_) => None,
        };

        let color_xy = match get_key(map, "color") {
            Ok(color) => {
                let color = color.as_object().ok_or(invalid_err("Object"))?;
                let color_x = get_key(color, "x")?
                    .as_number()
                    .ok_or(invalid_err("Number"))?
                    .as_f64()
                    .expect("Should be Some if not using arbitrary precision");
                let color_y = get_key(color, "y")?
                    .as_number()
                    .ok_or(invalid_err("Number"))?
                    .as_f64()
                    .expect("Should be Some if not using arbitrary precision");
                Some((color_x, color_y))
            }
            Err(_) => None,
        };

        let brightness = match get_key(map, "brightness") {
            Ok(bri) => {
                let bri: u8 = bri
                    .as_number()
                    .ok_or(invalid_err("Number"))?
                    .as_u64()
                    .ok_or(invalid_err("u64"))?
                    .try_into()
                    .map_err(|_| invalid_err("u8"))?;
                Some(bri)
            }
            Err(_) => None,
        };

        let on = match get_key(map, "state") {
            Ok(on) => {
                let state = on.as_str().ok_or(invalid_err("String"))?;
                let on = match state.to_lowercase().as_str() {
                    "on" => true,
                    "off" => false,
                    other => {
                        return Err(invalid_err(&format!(
                            "on/off bool: {other}"
                        )))
                    }
                };
                Some(on)
            }
            Err(_) => None,
        };

        Ok(Lamp {
            #[allow(clippy::cast_precision_loss)]
            brightness: brightness.map(normalize),
            color_temp_k: color_temp_mired.map(mired_to_kelvin),
            color_xy,
            on,
        })
    }
}

impl Lamp {
    pub(crate) fn apply(&self, change: Change) -> Lamp {
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

    pub(crate) fn to_payloads(&self, model: &Model) -> Vec<String> {
        let mut payloads = vec![];
        match self.on {
            Some(true) => payloads.push(json!({ "state": "ON" })),
            Some(false) => payloads.push(json!({ "state": "OFF" })),
            None => (),
        };

        match self.brightness {
            Some(bri) => {
                payloads.push(json!({ "brightness": denormalize(bri) }))
            }
            None => (),
        }

        if model.is_color_lamp() {
            if let Some(color_xy) = self.color_xy {
                payloads.push(
                    json!({ "color": {"x": color_xy.0, "y": color_xy.1} }),
                );
            }
        } else {
            if let Some(color_temp) = self.color_temp_k {
                payloads
                    .push(json!({ "color_temp": kelvin_to_mired(color_temp) }));
            }
        }

        payloads.into_iter().map(|x| x.to_string()).collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Change {
    On(bool),
    Brightness(f64),
    ColorTemp(usize),
    ColorXy((f64, f64)),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Model {
    TradfriCandle,
    TradfriE27,
    TradfriE14,
    HueGen4,
    TradfriOther(String),
    Other(String),
}

impl Model {
    pub(crate) fn is_color_lamp(&self) -> bool {
        use Model as M;
        match self {
            M::TradfriE27 | M::TradfriE14 | M::HueGen4 => true,
            M::TradfriCandle => false,
            // We assume no so that things at least don't break
            M::TradfriOther(_) | M::Other(_) => false,
        }
    }
}
