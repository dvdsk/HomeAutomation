use std::io;

use serde_json::Value;

use crate::lights::{lamp::LampState, mired_to_kelvin, normalize};

impl TryInto<LampState> for &[u8] {
    type Error = io::Error;

    fn try_into(self) -> Result<LampState, Self::Error> {
        let json: Value = serde_json::from_slice(self)?;
        let map = json.as_object().ok_or(invalid_err("Object"))?;

        let color_temp_mired = match get_key(map, "color_temp") {
            Ok(temp) => {
                let color_temp: usize =
                    json_to_u64(temp)?.try_into().expect("usize should be u64");
                Some(color_temp)
            }
            Err(_) => None,
        };

        let color_xy = match get_key(map, "color") {
            Ok(color) => {
                let color = color.as_object().ok_or(invalid_err("Object"))?;
                let color_x = json_to_f64(get_key(color, "x")?)?;
                let color_y = json_to_f64(get_key(color, "y")?)?;
                Some((color_x, color_y))
            }
            Err(_) => None,
        };

        let brightness = match get_key(map, "brightness") {
            Ok(bri) => {
                let bri: u8 = json_to_u64(bri)?
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

        Ok(LampState {
            #[allow(clippy::cast_precision_loss)]
            brightness: brightness.map(normalize),
            color_temp_k: color_temp_mired.map(mired_to_kelvin),
            color_xy,
            on,
        })
    }
}

fn json_to_u64(json: &Value) -> Result<u64, io::Error> {
    Ok(json
        .as_number()
        .ok_or(invalid_err("Number"))?
        .as_u64()
        .ok_or(invalid_err("u64"))?)
}

fn json_to_f64(json: &Value) -> Result<f64, io::Error> {
    Ok(json
        .as_number()
        .ok_or(invalid_err("Number"))?
        .as_f64()
        .expect("Should be Some if not using arbitrary precision"))
}

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
