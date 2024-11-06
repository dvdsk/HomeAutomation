use std::io;

use serde_json::{json, Value};

use crate::lights::{
    conversion::{normalize, temp_to_xy},
    denormalize,
};

#[derive(Debug, Clone)]
pub(crate) struct Lamp {
    pub(crate) brightness: f64,
    pub(crate) color_xy: Option<(f64, f64)>,
    pub(crate) on: bool,
}

impl Default for Lamp {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            color_xy: Some(temp_to_xy(2700)),
            on: false,
        }
    }
}

impl PartialEq for Lamp {
    fn eq(&self, other: &Self) -> bool {
        let d_bright = (self.brightness - other.brightness).abs();
        let (d_color_x, d_color_y) = match (self.color_xy, other.color_xy) {
            (Some(self_xy), Some(other_xy)) => (
                (self_xy.0 - other_xy.0).abs(),
                (self_xy.1 - other_xy.1).abs(),
            ),
            // If either Lamp has no xy set, they are different
            _ => return false,
        };

        self.on == other.on
            && d_bright < 1. / 250.
            && d_color_x < 0.01
            && d_color_y < 0.01
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

        let brightness: u8 = get_key(map, "brightness")?
            .as_number()
            .ok_or(invalid_err("Number"))?
            .as_u64()
            .ok_or(invalid_err("u64"))?
            .try_into()
            .map_err(|_| invalid_err("u8"))?;

        let state = get_key(map, "state")?
            .as_str()
            .ok_or(invalid_err("String"))?;
        let on = match state.to_lowercase().as_str() {
            "on" => true,
            "off" => false,
            other => return Err(invalid_err(&format!("on/off bool: {other}"))),
        };

        Ok(Lamp {
            #[allow(clippy::cast_precision_loss)]
            brightness: normalize(brightness),
            color_xy,
            on,
        })
    }
}

impl Lamp {
    pub(crate) fn apply(&self, change: Change) -> Lamp {
        let mut new_state = self.clone();
        match change {
            Change::On(on) => new_state.on = on,
            Change::Brightness(bri) => new_state.brightness = bri,
            Change::ColorTemp(temp) => {
                new_state.color_xy = Some(temp_to_xy(temp));
            }
            Change::ColorXy(xy) => new_state.color_xy = Some(xy),
        }
        new_state
    }

    pub(crate) fn to_payloads(&self) -> Vec<String> {
        let state = if self.on { "ON" } else { "OFF" };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mut payloads = vec![
            json!({ "state": state }),
            json!({ "brightness": denormalize(self.brightness) }),
        ];

        if let Some(color_xy) = self.color_xy {
            payloads.push(json!({ "color_xy": color_xy }));
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
