use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::{Context, OptionExt};
use color_eyre::Section;
use regex::Regex;
use rumqttc::v5::{Event, EventLoop, Incoming};
use serde_json::Value;
use tokio::{sync::RwLock, time::sleep};
use tracing::{instrument, trace, warn};

use crate::lights::denormalize;
use crate::lights::lamp::{self, Lamp};
use crate::lights::parse_state::parse_lamp_properties;

pub(super) async fn poll_mqtt(
    mut eventloop: EventLoop,
    known_states: &RwLock<HashMap<String, Lamp>>,
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
                warn!("ZB error parsing mqtt message: {err}");
                continue;
            }
        };

        match message {
            Message::StateUpdate((light_name, changed_properties)) => {
                update_state(known_states, &light_name, changed_properties)
                    .await;
            }
            Message::Other => (),
        }
    }
}

async fn update_state(
    known_states: &RwLock<HashMap<String, Lamp>>,
    light_name: &str,
    new: Vec<lamp::Property>,
) {
    let mut known_states = known_states.write().await;
    let current_lamp = known_states
        .entry(light_name.to_owned())
        .or_insert_with(|| Lamp::new(light_name));
    for property in new {
        if light_name == "kitchen:ceiling" {
            match property {
                lamp::Property::Brightness(bri) => {
                    warn!(
                        "ZB received ceiling brightness change: {}",
                        denormalize(bri)
                    );
                }
                lamp::Property::Online(new_online) => {
                    if new_online != current_lamp.is_online {
                        warn!(
                            "ZB received ceiling online change: {new_online}"
                        );
                    }
                }
                _ => (),
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

    let Incoming::Publish(message) = incoming else {
        return Ok(Message::Other);
    };

    trace!("message: {message:?}");
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
            let name = topic[1].to_string();
            let state = parse_lamp_properties(&message.payload)
                .wrap_err("failed to parse lamp state")
                .with_note(|| format!("topic: {topic:?}"))?;
            Ok(Message::StateUpdate((name, state)))
        }
    }
}

fn parse_bridge_event(
    payload: &serde_json::Map<String, Value>,
) -> color_eyre::Result<Message> {
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
    let light_name = data
        .get("friendly_name")
        .ok_or_eyre("no name in bridge event data")?
        .as_str()
        .ok_or_eyre("bridge event friendly name is not a string")?
        .to_owned();

    let is_online = match event {
        "device_joined" | "device_announce" => true,
        "device_leave" => false,
        _ => return Ok(Message::Other),
    };

    let update = (light_name, vec![lamp::Property::Online(is_online)]);
    Ok(Message::StateUpdate(update))
}

fn parse_log_message(
    log: &serde_json::Map<String, Value>,
) -> color_eyre::Result<Message> {
    let level = log.get("level").ok_or_eyre("no level in log message")?;
    let message = log
        .get("message")
        .ok_or_eyre("no message in log message")?
        .as_str()
        .ok_or_eyre("log message is not a string")?;

    if level != "error" {
        return Ok(Message::Other);
    }

    let regex = Regex::new(r"Publish.*? to '(.*?)' failed").unwrap();

    if level == "error" {
        if let Some(caps) = regex.captures(message) {
            let light_name = caps[1].to_string();
            let update = (light_name, vec![lamp::Property::Online(false)]);
            return Ok(Message::StateUpdate(update));
        }
    }

    Ok(Message::Other)
}

#[derive(Debug)]
enum Message {
    StateUpdate((String, Vec<lamp::Property>)),
    Other,
}
