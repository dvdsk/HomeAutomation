use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::{Context, ContextCompat, OptionExt};
use color_eyre::Section;
use itertools::Itertools;
use ratelimited_logger::RateLimitedLogger;
use rumqttc::v5::{Event, EventLoop, Incoming};
use serde_json::Value;
use tokio::{sync::RwLock, time::sleep};
use tracing::{instrument, trace, warn};

use crate::lights::kelvin_to_mired;
use crate::lights::lamp::{Lamp, LampProperty, Model};
use crate::lights::parse_state::parse_lamp_properties;

pub(super) async fn poll_mqtt(
    mut eventloop: EventLoop,
    known_states: &RwLock<HashMap<String, Lamp>>,
) -> ! {
    let mut logger = RateLimitedLogger::default();

    loop {
        let message = match eventloop.poll().await {
            Ok(message) => message,
            Err(err) => {
                trace!("Error while polling: {err}");
                sleep(Duration::from_millis(100)).await;
                continue;
            }
        };

        let message = match parse_message(message) {
            Ok(message) => message,
            Err(err) => {
                ratelimited_logger::warn!(logger; "ZB error parsing mqtt message: {err}");
                continue;
            }
        };

        match message {
            Message::StateUpdate((light_name, changed_properties)) => {
                update_state(known_states, light_name, changed_properties)
                    .await;
            }
            Message::Devices(new_devices) => {
                for (light_name, model) in new_devices {
                    update_model(known_states, light_name, model).await;
                }
            }
            Message::Other => (),
        }
    }
}

async fn update_model(
    known_states: &RwLock<HashMap<String, Lamp>>,
    light_name: String,
    model: Model,
) {
    let mut known_states = known_states.write().await;
    let curr_model = &mut known_states
        .entry(light_name.to_string())
        .or_default()
        .model;
    *curr_model = Some(model);
}

async fn update_state(
    known_states: &RwLock<HashMap<String, Lamp>>,
    light_name: String,
    new: Vec<LampProperty>,
) {
    let mut known_states = known_states.write().await;
    let curr = &mut known_states
        .entry(light_name.to_string())
        .or_default()
        .state;
    for property in new {
        match property {
            LampProperty::Brightness(bri) => curr.brightness = Some(bri),
            LampProperty::ColorTempK(temp) => {
                if light_name == "kitchen:fridge" {
                    warn!(
                        "ZB received fridge color temp change: {}",
                        kelvin_to_mired(temp)
                    );
                }
                curr.color_temp_k = Some(temp)
            }
            LampProperty::ColorXY(xy) => curr.color_xy = Some(xy),
            LampProperty::On(is_on) => curr.on = Some(is_on),
            LampProperty::ColorTempStartup(behavior) => {
                curr.color_temp_startup = behavior
            }
        }
    }
}

#[instrument(skip_all)]
fn parse_message(event: Event) -> color_eyre::Result<Message> {
    let Event::Incoming(incoming) = event else {
        return Ok(Message::Other);
    };

    let Incoming::Publish(message) = incoming else {
        return Ok(Message::Other);
    };

    trace!("message: {message:?}");
    let topic = String::from_utf8_lossy(&message.topic);

    if topic == "zigbee2mqtt/bridge/devices" {
        let json: Value = serde_json::from_slice(&message.payload)
            .wrap_err("could not parse message payload as json")?;
        let list = json
            .as_array()
            .ok_or_eyre("devices list should be array its not")
            .with_note(|| format!("json was: {json:?}"))?;
        let devices: HashMap<String, Model> = list
            .iter()
            .map(|dev| {
                parse_device(dev)
                    .wrap_err("could not parse device")
                    .with_note(|| format!("device: {dev:?}"))
            })
            .filter_ok(|d| d.0 != "Coordinator")
            .collect::<Result<_, _>>()?;
        Ok(Message::Devices(devices))
    } else {
        let topic = String::from_utf8_lossy(&message.topic);
        let topic: Vec<_> = topic.split('/').collect();
        let name = topic[1].to_string();
        let state = parse_lamp_properties(&message.payload)
            .wrap_err("failed to parse lamp state")
            .with_note(|| format!("topic: {topic:?}"))?;
        Ok(Message::StateUpdate((name, state)))
    }
}

#[derive(Debug)]
enum Message {
    StateUpdate((String, Vec<LampProperty>)),
    Devices(HashMap<String, Model>),
    Other,
}

#[instrument(skip_all)]
fn parse_device(device: &Value) -> color_eyre::Result<(String, Model)> {
    let properties = device
        .as_object()
        .wrap_err("device should be json object")?;

    let friendly_name = properties
        .get("friendly_name")
        .wrap_err("device misses key `friendly_name`")?
        .as_str()
        .wrap_err("value of `friendly_name` is not a string")?
        .to_owned();
    let model_id = properties
        .get("model_id")
        .map(Value::as_str)
        .map(|res| res.ok_or_eyre("value of `model_id` is not a string"))
        .transpose()?;

    let model = match model_id {
        Some("TRADFRI bulb E14 WS candle 470lm") => Model::TradfriCandle,
        Some("TRADFRI bulb E27 CWS globe 806lm") => Model::TradfriE27,
        Some("TRADFRI bulb E14 CWS globe 806lm") => Model::TradfriE14,
        Some("LCT001") => Model::HueGen4,
        Some(id) if id.to_lowercase().contains("tradfri") => {
            Model::TradfriOther(id.to_owned())
        }
        Some(id) => Model::Other(id.to_owned()),
        None => Model::Other(String::new()),
    };

    Ok((friendly_name, model))
}
