use color_eyre::eyre::{bail, Context, OptionExt, Report, Result};
use color_eyre::Section;
use protocol::small_bedroom;
use serde_json::{Map, Value};
use tracing::instrument;

use crate::conversion::{mired_to_kelvin, normalize};
use crate::device::Property;
use crate::lamp::LampProperty;
use crate::radiator::RadiatorProperty;
use crate::{light_names, RADIATOR_NAMES};

#[instrument(skip(map))]
pub(super) fn properties(
    device_name: &str,
    map: &Map<String, Value>,
) -> Result<Vec<Property>> {
    if light_names().contains(&device_name) {
        parse_lamp_properties(map)
    } else if RADIATOR_NAMES.contains(&device_name) {
        parse_radiator_properties(map)
    } else {
        Ok(Vec::new())
    }
}

pub(crate) fn portable_button_panel(
    device_name: &str,
    map: &Map<String, Value>,
) -> Result<Option<protocol::Reading>> {
    use protocol::small_bedroom::portable_button_panel::Reading as R;
    let action_reading_mapping = [
        ("toggle", R::PlayPause),
        ("track_next", R::TrackNext),
        ("track_previous", R::TrackPrevious),
        ("volume_up", R::VolumeUp),
        ("volume_up_hold", R::VolumeUpHold),
        ("volume_down", R::VolumeDown),
        ("volume_down_hold", R::VolumeDownHold),
        ("dots_1_initial_press", R::Dots1InitialPress),
        ("dots_1_short_release", R::Dots1ShortRelease),
        ("dots_1_double_press", R::Dots1DoublePress),
        ("dots_1_long_press", R::Dots1LongPress),
        ("dots_1_long_release", R::Dots1LongRelease),
        ("dots_2_initial_press", R::Dots2InitialPress),
        ("dots_2_short_release", R::Dots2ShortRelease),
        ("dots_2_double_press", R::Dots2DoublePress),
        ("dots_2_long_press", R::Dots2LongPress),
        ("dots_2_long_release", R::Dots2LongRelease),
    ];

    if device_name == "small_bedroom:portable_button_panel" {
        map.get("action")
            .map(json_to_str)
            .transpose()?
            .map(|action| {
                action_reading_mapping
                    .iter()
                    .find(|(a, _)| action == *a)
                    .map(|(_, reading)| reading.clone())
                    .map(small_bedroom::Reading::PortableButtonPanel)
                    .map(protocol::Reading::SmallBedroom)
                    .ok_or_eyre(
                        "every action should have a corrosponding reading",
                    )
                    .with_note(|| format!("action was: {action}"))
            })
            .transpose()
    } else {
        Ok(None)
    }
}

pub(crate) fn radiator_readings(
    device_name: &str,
    map: &Map<String, Value>,
) -> Result<Vec<protocol::Reading>> {
    use protocol::{large_bedroom, small_bedroom, Reading};

    macro_rules! radiator_readings {
        ($protocol_module:ident, $ReadingVariant:ident) => {
            Ok(std::iter::empty()
                .into_iter()
                .chain(
                    map.get("local_temperature")
                        .map(json_to_f32)
                        .transpose()?
                        .map($protocol_module::radiator::Reading::Temperature)
                        .map($protocol_module::Reading::Radiator)
                        .map(Reading::$ReadingVariant),
                )
                .chain(
                    map.get("pi_heating_demand")
                        .map(json_to_f32)
                        .transpose()?
                        .map($protocol_module::radiator::Reading::Heating)
                        .map($protocol_module::Reading::Radiator)
                        .map(Reading::$ReadingVariant),
                )
                .chain(
                    map.get("setpoint_change_source")
                        .map(json_to_str)
                        .transpose()?
                        .map(|val| match val.to_lowercase().as_str() {
                            "manual" => {
                                Ok($protocol_module::radiator::Source::Manual)
                            }
                            "schedule" => {
                                Ok($protocol_module::radiator::Source::Schedule)
                            }
                            "externally" => {
                                Ok($protocol_module::radiator::Source::External)
                            }
                            _ => Err(color_eyre::eyre::eyre!(
                                "Unexpected change source {val}"
                            )),
                        })
                        .transpose()?
                        .map($protocol_module::radiator::Reading::SetBy)
                        .map($protocol_module::Reading::Radiator)
                        .map(Reading::$ReadingVariant),
                )
                .chain(
                    map.get("occupied_heating_setpoint")
                        .map(json_to_f32)
                        .transpose()?
                        .map($protocol_module::radiator::Reading::Setpoint)
                        .map($protocol_module::Reading::Radiator)
                        .map(Reading::$ReadingVariant),
                )
                .collect())
        };
    }

    match device_name {
        "small_bedroom:radiator" => {
            radiator_readings!(small_bedroom, SmallBedroom)
        }
        "large_bedroom:radiator" => {
            radiator_readings!(large_bedroom, LargeBedroom)
        }
        _ => Ok(Vec::new()),
    }
}

fn parse_radiator_properties(
    map: &Map<String, Value>,
) -> Result<Vec<Property>> {
    let mut list = Vec::new();

    if let Some(setpoint) = map
        .get("occupied_heating_setpoint")
        .map(json_to_f64)
        .transpose()?
    {
        list.push(RadiatorProperty::Setpoint(setpoint).into());
    }

    if let Some(reference) = map
        .get("external_measured_room_sensor")
        .map(json_to_i64)
        .transpose()?
    {
        if reference == -8000 {
            list.push(RadiatorProperty::NoReference.into());
        } else {
            let reference = reference as f64 / 100.;
            list.push(RadiatorProperty::Reference(reference).into());
        }
    }

    if let Some(set_method) = map
        .get("setpoint_change_source")
        .map(|s| {
            s.as_str()
                .ok_or_eyre("Setpoint change source not a string")
                .map(|s| s.parse().unwrap())
        })
        .transpose()?
    {
        list.push(RadiatorProperty::SetByMethod(set_method).into());
    }

    Ok(list)
}

fn parse_lamp_properties(map: &Map<String, Value>) -> Result<Vec<Property>> {
    let mut list = Vec::new();

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

fn json_to_usize(json: &Value) -> Result<usize> {
    json_to_u64(json)?
        .try_into()
        .wrap_err("Should be a usize integer")
        .with_note(|| format!("got: {json:?}"))
}

fn json_to_u8(json: &Value) -> Result<u8> {
    json_to_u64(json)?
        .try_into()
        .wrap_err("Should be a 8 bit integer")
        .with_note(|| format!("got: {json:?}"))
}

fn json_to_u64(json: &Value) -> Result<u64> {
    json.as_number()
        .ok_or_eyre("Must be a number")
        .with_note(|| format!("got: {json:?}"))?
        .as_u64()
        .ok_or_eyre("Must be a positive integer")
        .with_note(|| format!("got: {json:?}"))
}

fn json_to_i64(json: &Value) -> Result<i64> {
    json.as_number()
        .ok_or_eyre("Must be a number")
        .with_note(|| format!("got: {json:?}"))?
        .as_i64()
        .ok_or_eyre("Must be an integer")
        .with_note(|| format!("got: {json:?}"))
}

fn json_to_f64(json: &Value) -> Result<f64> {
    Ok(json
        .as_number()
        .ok_or_eyre("Must be a number")
        .with_note(|| format!("got: {json:?}"))?
        .as_f64()
        .expect("Should be Some if not using arbitrary precision"))
}

fn json_to_str(json: &Value) -> Result<&str> {
    Ok(json
        .as_str()
        .ok_or_eyre("Must be a string")
        .with_note(|| format!("got: {json:?}"))?)
}

fn json_to_f32(json: &Value) -> Result<f32> {
    json_to_f64(json).map(|v| v as f32)
}
