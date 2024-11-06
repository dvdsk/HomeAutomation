use std::{collections::HashMap, time::Duration};

use rumqttc::{Event, EventLoop, Incoming};
use serde_json::Value;
use tokio::{sync::RwLock, time::sleep};

use crate::lights::state::Lamp;

pub(super) async fn poll_mqtt(
    mut eventloop: EventLoop,
    known_states: &RwLock<HashMap<String, Lamp>>,
    devices: &RwLock<HashMap<String, Model>>,
) -> ! {
    loop {
        let message = match eventloop.poll().await {
            Ok(message) => message,
            Err(err) => {
                println!("Error while polling: {err}");
                sleep(Duration::from_millis(100)).await;
                continue;
            }
        };

        match parse_message(message) {
            Message::StateUpdate((light_name, new_known_state)) => {
                let mut known_states = known_states.write().await;
                known_states.insert(light_name, new_known_state);
            }
            Message::Devices(new_devices) => {
                let mut devices = devices.write().await;
                if *devices != new_devices {
                    dbg!(&new_devices);
                }
                *devices = new_devices
            }
            Message::Other => (),
        }
    }
}

fn parse_message(message: Event) -> Message {
    match message {
        Event::Incoming(incoming) => match incoming {
            Incoming::ConnAck(_)
            | Incoming::PubAck(_)
            | Incoming::PingResp
            | Incoming::SubAck(_) => Message::Other,
            Incoming::Publish(message)
                if message.topic == "zigbee2mqtt/bridge/devices" =>
            {
                let json: Value =
                    serde_json::from_slice(&message.payload).unwrap();
                let list = json.as_array().unwrap();
                let devices: HashMap<String, Model> = list
                    .into_iter()
                    .map(parse_device)
                    .filter(|d| d.0 != "Coordinator")
                    .collect();

                Message::Devices(devices)
            }
            Incoming::Publish(message) => {
                let topic: Vec<_> = message.topic.split('/').collect();
                let name = topic[1].to_string();
                let data = &(*message.payload);

                Message::StateUpdate((name, data.try_into().unwrap()))
            }
            other => {
                dbg!(other);
                Message::Other
            }
        },
        Event::Outgoing(_) => Message::Other,
    }
}

#[derive(Debug)]
enum Message {
    StateUpdate((String, Lamp)),
    Devices(HashMap<String, Model>),
    Other,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Model {
    TradfriCandle,
    TradfriE27,
    TradfriE14,
    HueGen4,
    TradfriOther(String),
    Other(String),
}

fn parse_device(device: &Value) -> (String, Model) {
    let properties = device.as_object().unwrap();

    let friendly_name = properties
        .get("friendly_name")
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned();
    let model_id = properties
        .get("model_id")
        .map(Value::as_str)
        .map(Option::unwrap);

    let model = match model_id {
        Some("TRADFRI bulb E14 WS candle 470lm") => Model::TradfriCandle,
        Some("TRADFRI bulb E27 CWS globe 806lm") => Model::TradfriE27,
        Some("TRADFRI bulb E14 CWS globe 806lm") => Model::TradfriE14,
        Some("LCT001") => Model::HueGen4,
        Some(id) if id.to_lowercase().contains("tradfri") => {
            Model::TradfriOther(id.to_owned())
        }
        Some(id) => Model::Other(id.to_owned()),
        None => Model::Other("".to_owned()),
    };

    (friendly_name, model)
}
