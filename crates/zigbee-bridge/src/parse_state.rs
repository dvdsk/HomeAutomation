use color_eyre::eyre::{bail, Context, OptionExt, Report};
use color_eyre::Section;
use serde_json::Value;
use tracing::error;

use crate::conversion::{mired_to_kelvin, normalize};
use crate::device::Property;
use crate::lamp::LampProperty;
use crate::{light_names, RADIATOR_NAMES};

pub(super) fn parse_properties(
    device_name: &str,
    bytes: &[u8],
) -> color_eyre::Result<Vec<Property>> {
    if light_names().contains(&device_name) {
        parse_lamp_properties(bytes)
    } else if RADIATOR_NAMES.contains(&device_name) {
        parse_radiator_properties(bytes)
    } else {
        error!("Unknown device name {device_name}, could not parse properties");
        // TODO make actual error
        return Ok(Vec::new());
    }
}

fn parse_radiator_properties(bytes: &[u8]) -> Result<Vec<Property>, Report> {
    Ok(Vec::new())
}

fn parse_lamp_properties(
    bytes: &[u8],
) -> color_eyre::Result<Vec<Property>> {
    let mut list = Vec::new();

    let json: Value =
        serde_json::from_slice(bytes).wrap_err("Could not deserialize")?;
    let map = json
        .as_object()
        .ok_or_eyre("Top level json must be object")?;

    if let Some(kelvin) = map
        .get("color_temp")
        .map(json_to_usize)
        .transpose()?
        .map(mired_to_kelvin)
    {
        list.push(LampProperty::ColorTempK(kelvin).into());
    }

    if let Some(xy) = map
        .get("color")
        .map(|color| {
            let color = color
                .as_object()
                .ok_or_eyre("Color json should be an object")?;
            let color_x =
                json_to_f64(color.get("x").ok_or_eyre("Need a key 'x'")?)?;
            let color_y =
                json_to_f64(color.get("y").ok_or_eyre("Need a key 'y'")?)?;
            Ok::<_, Report>((color_x, color_y))
        })
        .transpose()?
    {
        list.push(LampProperty::ColorXY(xy).into());
    }

    if let Some(brightness) = map
        .get("brightness")
        .map(json_to_u8)
        .transpose()?
        .map(normalize)
    {
        list.push(LampProperty::Brightness(brightness).into());
    }

    if let Some(on) = map
        .get("state")
        .map(|on| {
            let state = on.as_str().ok_or_eyre("state should be a string")?;
            let on = match state {
                "ON" => true,
                "OFF" => false,
                _ => bail!("state string should be ON or OFF"),
            };
            Ok(on)
        })
        .transpose()?
    {
        list.push(LampProperty::On(on).into());
    }

    // we have just received a state message, so the lamp must be online
    list.push(LampProperty::Online(true).into());

    Ok(list)
}

fn json_to_usize(json: &Value) -> color_eyre::Result<usize> {
    json_to_u64(json)?
        .try_into()
        .wrap_err("Should be a usize integer")
        .with_note(|| format!("got: {json:?}"))
}

fn json_to_u8(json: &Value) -> color_eyre::Result<u8> {
    json_to_u64(json)?
        .try_into()
        .wrap_err("Should be a 8 bit integer")
        .with_note(|| format!("got: {json:?}"))
}

fn json_to_u64(json: &Value) -> color_eyre::Result<u64> {
    json.as_number()
        .ok_or_eyre("Must be a number")
        .with_note(|| format!("got: {json:?}"))?
        .as_u64()
        .ok_or_eyre("Must be a positive integer")
        .with_note(|| format!("got: {json:?}"))
}

fn json_to_f64(json: &Value) -> color_eyre::Result<f64> {
    Ok(json
        .as_number()
        .ok_or_eyre("Must be a number")
        .with_note(|| format!("got: {json:?}"))?
        .as_f64()
        .expect("Should be Some if not using arbitrary precision"))
}
