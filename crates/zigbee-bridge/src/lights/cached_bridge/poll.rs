use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::{Context, ContextCompat, OptionExt};
use color_eyre::Section;
use itertools::Itertools;
use ratelimited_logger::RateLimitedLogger;
use rumqttc::v5::{Event, EventLoop, Incoming};
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::{sync::RwLock, time::sleep};
use tracing::{instrument, trace, warn};

use crate::lights::kelvin_to_mired;
use crate::lights::lamp::{self, Lamp};
use crate::lights::parse_state::parse_lamp_properties;

use super::mqtt::Mqtt;

pub(super) async fn poll_mqtt(
    mut eventloop: EventLoop,
    mqtt: &RwLock<Mqtt>,
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
                warn!("Received devices list");
                for (light_name, model) in new_devices {
                    update_model(known_states, light_name, model).await;
                }
            }
            Message::ConnAck => {
                warn!("Received ConnAck, resubscribing");
                let mqtt = mqtt.read().await;
                mqtt.subscribe_to_all().await;
            }
            Message::Other => (),
        }
    }
}

async fn update_model(
    known_states: &RwLock<HashMap<String, Lamp>>,
    light_name: String,
    model: lamp::Model,
) {
    let mut known_states = known_states.write().await;
    let current_lamp = known_states.entry(light_name.to_string()).or_default();
    if light_name == "kitchen:fridge" {
        warn!(
            "Received model update for fridge lamp,\nold: {:?},\nnew: {model:?}",
            current_lamp.model
            );
    }
    current_lamp.set_model(model);
}

async fn update_state(
    known_states: &RwLock<HashMap<String, Lamp>>,
    light_name: String,
    new: Vec<lamp::Property>,
) {
    let mut known_states = known_states.write().await;
    let current_lamp = known_states.entry(light_name.to_string()).or_default();
    for property in new {
        if let lamp::Property::ColorTempK(temp) = property {
            if light_name == "kitchen:fridge" {
                warn!(
                    "ZB received fridge color temp change: {}",
                    kelvin_to_mired(temp)
                );
            }
        }
        current_lamp.apply(property);
    }
}

#[instrument(skip_all)]
fn parse_message(event: Event) -> color_eyre::Result<Message> {
    let Event::Incoming(incoming) = event else {
        return Ok(Message::Other);
    };

    let message = match incoming {
        Incoming::Publish(message) => message,
        Incoming::ConnAck(_) => return Ok(Message::ConnAck),
        _ => return Ok(Message::Other),
    };

    trace!("message: {message:?}");
    let topic: &str = &String::from_utf8_lossy(&message.topic);

    match topic {
        "zigbee2mqtt/bridge/devices" => {
            warn!("ZB received devices message");
            let json: Value = serde_json::from_slice(&message.payload)
                .wrap_err("could not parse message payload as json")?;
            warn!("contents: {json:#?}");
            let list = json
                .as_array()
                .ok_or_eyre("devices list should be array its not")
                .with_note(|| format!("json was: {json:?}"))?;
            let devices: HashMap<String, lamp::Model> = list
                .iter()
                .map(|dev| {
                    parse_device(dev)
                        .wrap_err("could not parse device")
                        .with_note(|| format!("device: {dev:?}"))
                })
                .filter_ok(|d| d.0 != "Coordinator")
                .collect::<Result<_, _>>()?;
            Ok(Message::Devices(devices))
        }
        "zigbee2mqtt/bridge/logging" => {
            let json: Value = serde_json::from_slice(&message.payload)
                .wrap_err("could not parse message payload as json")?;
            let log = json
                .as_object()
                .ok_or_eyre("log should be map it is not")
                .with_note(|| format!("json was: {json:?}"))?;
            // parse_log_message(log)
            Ok(Message::Other)
        }
        topic => {
            let topic: Vec<_> = topic.split('/').collect();
            let name = topic[1].to_string();
            let state = parse_lamp_properties(&message.payload)
                .wrap_err("failed to parse lamp state")
                .with_note(|| format!("topic: {topic:?}"))?;
            Ok(Message::StateUpdate((name, state)))
        }
    }
}

fn parse_log_message(
    log: &serde_json::Map<String, Value>,
) -> color_eyre::Result<Message> {
    let level = log.get("level").ok_or_eyre("no level in log message")?;
    let message = log.get("message").ok_or_eyre("no message in log message")?;
    if level == "error" {
        todo!()
    } else {
        Ok(Message::Other)
    }
}

#[derive(Debug)]
enum Message {
    StateUpdate((String, Vec<lamp::Property>)),
    Devices(HashMap<String, lamp::Model>),
    ConnAck,
    Other,
}

#[instrument(skip_all)]
fn parse_device(device: &Value) -> color_eyre::Result<(String, lamp::Model)> {
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
        Some("TRADFRI bulb E14 WS candle 470lm") => lamp::Model::TradfriCandle,
        Some("TRADFRI bulb E27 CWS globe 806lm") => lamp::Model::TradfriE27,
        Some("TRADFRI bulb E14 CWS globe 806lm") => lamp::Model::TradfriE14,
        Some("LCT001") => lamp::Model::HueGen4,
        Some(id) if id.to_lowercase().contains("tradfri") => {
            lamp::Model::TradfriOther(id.to_owned())
        }
        Some(id) => lamp::Model::Other(id.to_owned()),
        None => lamp::Model::Other(String::new()),
    };

    Ok((friendly_name, model))
}
