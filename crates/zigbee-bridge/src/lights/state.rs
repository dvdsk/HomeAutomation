#![allow(unused)]

use std::io;

use serde_json::Value;

use crate::lights::conversion::{mired_to_kelvin, temp_to_xy};

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct State {
    pub(crate) brightness: f64,
    pub(crate) color_temp_kelvin: usize,
    pub(crate) color_xy: (f64, f64),
    pub(crate) on: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            color_temp_kelvin: 2700,
            color_xy: temp_to_xy(2700),
            on: false,
        }
    }
}

impl TryInto<State> for &[u8] {
    type Error = io::Error;

    fn try_into(self) -> Result<State, Self::Error> {
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

        let json: Value = serde_json::from_slice(&self)?;
        let map = json.as_object().ok_or(invalid_err("Object"))?;

        let color = get_key(map, "color")?
            .as_object()
            .ok_or(invalid_err("Object"))?;
        let color_x = get_key(color, "x")?
            .as_number()
            .ok_or(invalid_err("Number"))?
            .as_f64()
            .expect("Should return Some if not using arbitrary precision");
        let color_y = get_key(color, "y")?
            .as_number()
            .ok_or(invalid_err("Number"))?
            .as_f64()
            .expect("Should return Some if not using arbitrary precision");

        let brightness = get_key(map, "brightness")?
            .as_number()
            .ok_or(invalid_err("Number"))?
            .as_u64()
            .ok_or(invalid_err("u64"))?;

        let color_temp_mired = get_key(map, "color_temp")?
            .as_number()
            .ok_or(invalid_err("Number"))?
            .as_u64()
            .ok_or(invalid_err("u64"))?
            .try_into()
            .expect("usize should be u64");

        let state = get_key(map, "state")?
            .as_str()
            .ok_or(invalid_err("String"))?;
        let on = match state.to_lowercase().as_str() {
            "on" => true,
            "off" => false,
            other => return Err(invalid_err("on/off bool")),
        };

        Ok(State {
            brightness: brightness as f64 / 254.,
            color_temp_kelvin: mired_to_kelvin(color_temp_mired),
            color_xy: (color_x, color_y),
            on,
        })
    }
}

impl State {
    pub(crate) fn apply(&mut self, change: Change) -> State {
        todo!()
    }
}

#[derive(Debug)]
// TODO: make enum
pub(crate) struct Change {
    pub(crate) friendly_name: String,
    pub(crate) on: Option<bool>,
    pub(crate) brightness: Option<f64>,
    pub(crate) color_temp: Option<usize>,
    pub(crate) color_xy: Option<(f64, f64)>,
}
