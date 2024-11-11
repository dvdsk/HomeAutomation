use std::io;

use serde_json::{json, Value};

use crate::lights::{
    conversion::{kelvin_to_mired, mired_to_kelvin, normalize, temp_to_xy},
    denormalize,
};

#[derive(Debug, Clone)]
pub(crate) struct Lamp {
    pub(crate) brightness: f64,
    pub(crate) color_temp_kelvin: usize,
    pub(crate) color_xy: (f64, f64),
    pub(crate) on: bool,
}

impl Default for Lamp {
    fn default() -> Self {
        Self {
            brightness: 1.0,
            color_temp_kelvin: 2700,
            color_xy: temp_to_xy(2700),
            on: false,
        }
    }
}

impl PartialEq for Lamp {
    fn eq(&self, other: &Self) -> bool {
        let d_bright = (self.brightness - other.brightness).abs();
        let d_color_temp = self.color_temp_kelvin - other.color_temp_kelvin;
        let d_color_x = (self.color_xy.0 - other.color_xy.0).abs();
        let d_color_y = (self.color_xy.1 - other.color_xy.1).abs();

        self.on == other.on
            && d_bright < 1. / 250.
            && d_color_temp < 20
            && d_color_x < 0.01
            && d_color_y < 0.01
    }
}

impl Eq for Lamp {}

impl TryInto<Lamp> for &[u8] {
    type Error = io::Error; // TODO: could be a parse error enum, saves having
                            // to go through the whole ErrorKind stuff.
                            //
                            // But this is fine too, kinda like the str for
                            // invalid_err() stuff
                            //
                            // but get_key(map, "color") could be
                            // map.get("color").ok_or(ParseError("Missing key
                            // from map: {key}")). Just an idea, do what you
                            // prefer.
                            // <11-11-24, dvdsk>

    fn try_into(self) -> Result<Lamp, Self::Error> {
        fn get_key<'a>(
            map: &'a serde_json::Map<String, Value>,
            key: &str,
        ) -> Result<&'a Value, io::Error> {
            // FIX: this is the literal string {key} (missing a format! call
            // probably?) <11-11-24, dvdsk>
            let key_err = io::Error::new(io::ErrorKind::InvalidData, "Missing key from map: {key}");
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

        let brightness: u8 = get_key(map, "brightness")?
            .as_number()
            .ok_or(invalid_err("Number"))?
            .as_u64()
            .ok_or(invalid_err("u64"))?
            .try_into()
            .map_err(|_| invalid_err("u8"))?;

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
        // TODO: why not compare to uppercase ON and OFF? Now this is confusing
        // because we set the state using uppercase ON and OFF in `to_payloads`
        // <11-11-24, dvdsk> 
        let on = match state.to_lowercase().as_str() {
            "on" => true,
            "off" => false,
            other => return Err(invalid_err(&format!("on/off bool: {other}"))),
        };

        Ok(Lamp {
            #[allow(clippy::cast_precision_loss)]
            brightness: normalize(brightness),
            color_temp_kelvin: mired_to_kelvin(color_temp_mired),
            color_xy: (color_x, color_y),
            on,
        })
    }
}

impl Lamp {
    // TODO: How do you feel about letting this take ownership of self. Then the
    // clone would move to the caller and the first parameter would take be &self
    // but self <11-11-24, dvdsk>
    pub(crate) fn apply(&self, change: Change) -> Lamp {
        let mut new_state = self.clone();
        match change {
            Change::On(on) => new_state.on = on,
            Change::Brightness(bri) => new_state.brightness = bri,
            Change::ColorTemp(temp) => new_state.color_temp_kelvin = temp,
            Change::ColorXy(xy) => new_state.color_xy = xy,
        }
        new_state
    }

    pub(crate) fn to_payloads(&self) -> Vec<String> {
        let state = if self.on { "ON" } else { "OFF" };
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let color_temp_mired = kelvin_to_mired(self.color_temp_kelvin);

        [
            json!({ "state": state }),
            json!({ "brightness": denormalize(self.brightness) }),
            json!({ "color_temp": color_temp_mired }),
            // TODO: make sure this doesn't override temp / always use this
            // json!({ "color_xy": self.color_xy }),
        ]
        .into_iter()
        .map(|x| x.to_string())
        .collect()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Change {
    On(bool),
    Brightness(f64),
    ColorTemp(usize),
    ColorXy((f64, f64)),
}
