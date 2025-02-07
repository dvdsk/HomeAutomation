use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::{Context, OptionExt};
use color_eyre::{Result, Section};
use regex::Regex;
use rumqttc::v5::{Event, EventLoop, Incoming};
use serde_json::Value;
use tokio::{sync::RwLock, time::sleep};
use tracing::{error, instrument, trace, warn};

use crate::device::{Device, Property};
use crate::lamp::LampProperty;
use crate::parse;
use crate::radiator::RadiatorProperty;
use crate::{light_names, RADIATOR_NAMES};

pub(super) async fn poll_mqtt(
    mut eventloop: EventLoop,
    known_states: &RwLock<HashMap<String, Box<dyn Device>>>,
    reading_callback: impl Fn(protocol::Reading),
) -> ! {
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
                warn!("ZB error parsing mqtt message: {err:?}");
                continue;
            }
        };

        match message {
            Message::Update {
                device_name,
                changed,
                readings,
            } => {
                for reading in readings {
                    reading_callback(reading);
                }
                update_state(known_states, &device_name, changed).await;
            }
            Message::Irrelevant => (),
        }
    }
}

#[instrument(skip(known_states))]
async fn update_state(
    known_states: &RwLock<HashMap<String, Box<dyn Device>>>,
    device_name: &str,
    new: Vec<Property>,
) {
    let mut known_states = known_states.write().await;
    let Some(current_device) = known_states.get_mut(device_name) else {
        if device_name != "small_bedroom:portable_button_panel" {
            error!("Trying to update state for unknown device {device_name}, ignoring!");
        }
        return;
    };
    trace!("applying properties");
    for property in new {
        current_device.apply(property);
    }
}

#[instrument(skip_all)]
fn parse_message(event: Event) -> Result<Message> {
    let Event::Incoming(incoming) = event else {
        return Ok(Message::Irrelevant);
    };

    let Incoming::Publish(message) = incoming else {
        return Ok(Message::Irrelevant);
    };

    let topic: &str = &String::from_utf8_lossy(&message.topic);
    match topic {
        "zigbee2mqtt/bridge/event" => {
            let json: Value = serde_json::from_slice(&message.payload)
                .wrap_err("could not parse message payload as json")?;
            let bridge_event = json
                .as_object()
                .ok_or_eyre("log should be map it is not")
                .with_note(|| format!("json was: {json:?}"))?;
            parse_bridge_event(bridge_event)
        }
        "zigbee2mqtt/bridge/logging" => {
            let json: Value = serde_json::from_slice(&message.payload)
                .wrap_err("could not parse message payload as json")?;
            let log = json
                .as_object()
                .ok_or_eyre("log should be map it is not")
                .with_note(|| format!("json was: {json:?}"))?;
            parse_log_message(log)
        }
        topic => {
            let topic: Vec<_> = topic.split('/').collect();
            let device_name = topic[1].to_string();
            let json: Value = serde_json::from_slice(&message.payload)
                .wrap_err("Could not deserialize")?;
            let map = json
                .as_object()
                .ok_or_eyre("Top level json must be object")?;
            let changed = parse::properties(&device_name, map)
                .wrap_err("failed to parse device state")
                .with_note(|| format!("topic: {topic:?}"))?;
            let mut readings = parse::radiator_readings(&device_name, map)
                .wrap_err("failed to parse device readings")
                .with_note(|| format!("topic: {topic:?}"))?;
            let button_readings =
                parse::portable_button_panel(&device_name, map)
                    .wrap_err("failed to parse media buttons")
                    .with_note(|| format!("topic: {topic:?}"))?;
            readings.extend(button_readings);
            Ok(Message::Update {
                device_name,
                changed,
                readings,
            })
        }
    }
}

fn parse_bridge_event(
    payload: &serde_json::Map<String, Value>,
) -> Result<Message> {
    let event = payload
        .get("type")
        .ok_or_eyre("no type in bridge event")?
        .as_str()
        .ok_or_eyre("bridge event type is not a string")?;
    let data = payload
        .get("data")
        .ok_or_eyre("no data in bridge event")?
        .as_object()
        .ok_or_eyre("bridge event data is not a map")?;
    let device_name = data
        .get("friendly_name")
        .ok_or_eyre("no name in bridge event data")?
        .as_str()
        .ok_or_eyre("bridge event friendly name is not a string")?
        .to_owned();

    let is_online = match event {
        "device_joined" | "device_announce" => true,
        "device_leave" => false,
        _ => return Ok(Message::Irrelevant),
    };

    Ok(online_message(device_name, is_online))
}

fn parse_log_message(log: &serde_json::Map<String, Value>) -> Result<Message> {
    let level = log.get("level").ok_or_eyre("no level in log message")?;
    let message = log
        .get("message")
        .ok_or_eyre("no message in log message")?
        .as_str()
        .ok_or_eyre("log message is not a string")?;

    if level != "error" {
        return Ok(Message::Irrelevant);
    }

    let regex = Regex::new(r"Publish.*? to '(.*?)' failed").unwrap();

    if level == "error" {
        if let Some(caps) = regex.captures(message) {
            let device_name = caps[1].to_string();

            return Ok(online_message(device_name, false));
        }
    }

    Ok(Message::Irrelevant)
}

fn online_message(device_name: String, is_online: bool) -> Message {
    if light_names().contains(&device_name.as_str()) {
        Message::Update {
            device_name,
            changed: vec![LampProperty::Online(is_online).into()],
            readings: Vec::new(),
        }
    } else if RADIATOR_NAMES.contains(&device_name.as_str()) {
        Message::Update {
            device_name,
            changed: vec![RadiatorProperty::Online(is_online).into()],
            readings: Vec::new(),
        }
    } else {
        if device_name != "small_bedroom:portable_button_panel" {
            error!("Unknown device name {device_name}, could not parse log");
        }
        Message::Irrelevant
    }
}

#[derive(Debug)]
enum Message {
    Update {
        device_name: String,
        changed: Vec<Property>,
        readings: Vec<protocol::Reading>,
    },
    Irrelevant,
}
