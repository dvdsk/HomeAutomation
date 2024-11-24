use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::{Context, OptionExt};
use color_eyre::Section;
use ratelimited_logger::RateLimitedLogger;
use rumqttc::v5::{Event, EventLoop, Incoming};
use serde_json::Value;
use tokio::{sync::RwLock, time::sleep};
use tracing::{instrument, trace, warn};

use crate::lights::kelvin_to_mired;
use crate::lights::lamp::{self, Lamp};
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

    let Incoming::Publish(message) = incoming else {
        return Ok(Message::Other);
    };

    trace!("message: {message:?}");
    let topic: &str = &String::from_utf8_lossy(&message.topic);

    match topic {
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
    Other,
}
