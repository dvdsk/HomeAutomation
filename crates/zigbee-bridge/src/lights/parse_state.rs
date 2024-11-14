use serde_json::Value;
use thiserror::Error;

use crate::lights::{lamp::LampState, mired_to_kelvin, normalize};

use super::lamp::LampProperty;

#[derive(Error, Debug)]
pub(super) enum Error {
    #[error("Could not deserialize to json: {0}")]
    NotJson(#[from] serde_json::Error),
    #[error("Needed key {0}, is missing")]
    MissingKey(&'static str),
    #[error("Needed a number however got: {0}")]
    NotNumber(String),
    #[error("Needed an integer number however got: {0}")]
    NotInteger(String),
    #[error("Needed a 8 bit integer number however got: {0}")]
    NumberNotU8(String),
    #[error("Needed an usize bit integer number however got: {0}")]
    NumberNotUsize(String),
    #[error("Needed an uf8 string got: {0}")]
    NotString(String),
    #[error("Needed json object got: {0}")]
    NotObject(String),
    #[error("Invalid light state, expected ON or EFF got: {0}")]
    InvalidState(String),
}

pub(super) fn parse_lamp_properties(bytes: &[u8]) -> Result<Vec<LampProperty>, Error> {
    let mut list = Vec::new();

    let json: Value = serde_json::from_slice(bytes)?;
    let map = json.as_object().ok_or(Error::NotObject(json.to_string()))?;

    if let Some(kelvin) = map
        .get("color_temp")
        .map(|mired| json_to_usize(mired))
        .transpose()?
        .map(mired_to_kelvin)
    {
        list.push(LampProperty::ColorTempK(kelvin));
    }

    if let Some(xy) = map
        .get("color")
        .map(|color| {
            let color = color
                .as_object()
                .ok_or(Error::NotObject(color.to_string()))?;
            let color_x = json_to_f64(color.get("x").ok_or(Error::MissingKey("x"))?)?;
            let color_y = json_to_f64(color.get("y").ok_or(Error::MissingKey("y"))?)?;
            Ok::<_, Error>((color_x, color_y))
        })
        .transpose()?
    {
        list.push(LampProperty::ColorXY(xy));
    }

    if let Some(brightness) = map
        .get("brightness")
        .map(|bri| json_to_u8(bri))
        .transpose()?
        .map(normalize)
    {
        list.push(LampProperty::Brightness(brightness));
    }

    if let Some(on) = map
        .get("state")
        .map(|on| {
            let state = on.as_str().ok_or(Error::NotString(on.to_string()))?;
            let on = match state {
                "ON" => true,
                "OFF" => false,
                other => return Err(Error::InvalidState(other.to_string())),
            };
            Ok(on)
        })
        .transpose()?
    {
        list.push(LampProperty::On(on))
    }

    // TODO: startup behavior? <14-11-24, dvdsk> 
    //
    Ok(list)
}

fn json_to_usize(json: &Value) -> Result<usize, Error> {
    json_to_u64(json)?
        .try_into()
        .map_err(|_| Error::NumberNotUsize(json.to_string()))
}

fn json_to_u8(json: &Value) -> Result<u8, Error> {
    json_to_u64(json)?
        .try_into()
        .map_err(|_| Error::NumberNotU8(json.to_string()))
}

fn json_to_u64(json: &Value) -> Result<u64, Error> {
    json.as_number()
        .ok_or(Error::NotNumber(json.to_string()))?
        .as_u64()
        .ok_or(Error::NotInteger(json.to_string()))
}

fn json_to_f64(json: &Value) -> Result<f64, Error> {
    Ok(json
        .as_number()
        .ok_or(Error::NotNumber(json.to_string()))?
        .as_f64()
        .expect("Should be Some if not using arbitrary precision"))
}
