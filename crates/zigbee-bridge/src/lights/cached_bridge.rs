use std::collections::HashMap;
use std::time::Duration;

use rumqttc::{AsyncClient, ClientError, EventLoop};
use rumqttc::{ConnectionError, Event};
use rumqttc::{Incoming, MqttOptions};
use serde_json::json;
use tokio::sync::{mpsc, RwLock};

use crate::lights::state::{Change, State};
use crate::{LIGHTS, MQTT_IP, MQTT_PORT, QOS};

const CHANGE_TIMEOUT: Duration = Duration::from_secs(5);

struct Mqtt {
    client: AsyncClient,
}

impl Mqtt {
    async fn send_new_state(
        &self,
        light_name: &str,
        needed_state: &State,
    ) -> Result<(), ClientError> {
        for payload in needed_state.to_payloads() {
            self.set(light_name, &payload).await?;
        }
        Ok(())
    }

    async fn set(
        &self,
        friendly_name: &str,
        payload: &str,
    ) -> Result<(), ClientError> {
        let topic = format!("zigbee2mqtt/{friendly_name}/set");

        self.publish(&topic, payload).await?;
        Ok(())
    }

    async fn get(
        &self,
        friendly_name: &str,
        payload: &str,
    ) -> Result<(), ClientError> {
        let topic = format!("zigbee2mqtt/{friendly_name}/get");

        self.publish(&topic, payload).await?;
        Ok(())
    }

    async fn publish(
        &self,
        topic: &str,
        payload: &str,
    ) -> Result<(), ClientError> {
        self.client.publish(topic, QOS, false, payload).await
    }

    async fn request_state(&self, name: &str) {
        println!("Requesting state for light {name}");
        let payload = json!({"state": ""});

        self.get(name, &payload.to_string()).await.unwrap();
    }

    async fn subscribe(&self, topic: &str) -> Result<(), ClientError> {
        self.client.subscribe(topic, QOS).await
    }
}

pub(crate) async fn run(
    mut change_receiver: mpsc::UnboundedReceiver<(String, Change)>,
) {
    let mut options =
        MqttOptions::new("ha-lightcontroller", MQTT_IP, MQTT_PORT);
    // Set incoming to max mqtt packet size, outgoing to rumqtt default
    options.set_max_packet_size(2_usize.pow(28), 10240);

    let known_states = RwLock::new(HashMap::new());
    let mut needed_states = HashMap::new();

    // Reconnecting to broker is handled by Eventloop::poll
    let (client, eventloop) = AsyncClient::new(options.clone(), 128);
    let mqtt = Mqtt { client };

    for light in LIGHTS {
        mqtt.subscribe(&format!("zigbee2mqtt/{light}"))
            .await
            .unwrap();
        mqtt.request_state(light).await;
    }

    let poll_mqtt = poll_mqtt(eventloop, &known_states);
    let handle_changes = handle_changes(
        &mut change_receiver,
        &mqtt,
        &known_states,
        &mut needed_states,
    );

    tokio::select! {
        _ = handle_changes => (),
        err = poll_mqtt =>
            println!("Something went wrong with the mqtt connection: {err:?}"),
    };
}

async fn poll_mqtt(
    mut eventloop: EventLoop,
    known_states: &RwLock<HashMap<String, State>>,
) -> Result<(), ConnectionError> {
    loop {
        let message = match eventloop.poll().await {
            Ok(message) => message,
            Err(err) => {
                println!("Error while polling: {err}");
                continue;
            }
        };

        if let Some((light_name, new_known_state)) =
            extract_state_update(message)
        {
            let mut known_states = known_states.write().await;
            known_states.insert(light_name, new_known_state);
        }
    }
}

async fn handle_changes(
    change_receiver: &mut mpsc::UnboundedReceiver<(String, Change)>,
    mqtt: &Mqtt,
    known_states: &RwLock<HashMap<String, State>>,
    needed_states: &mut HashMap<String, State>,
) -> ! {
    // Give the initial known states a chance to be fetched
    tokio::time::sleep(Duration::from_millis(500)).await;
    loop {
        match tokio::time::timeout(CHANGE_TIMEOUT, change_receiver.recv()).await
        {
            Ok(change) => {
                let (light_name, change) =
                    change.expect("Channel should never close");
                let known_states = known_states.read().await;

                let previous_needed_state = match needed_states.get(&light_name)
                {
                    Some(needed_state) => needed_state,
                    None => match known_states.get(&light_name) {
                        Some(known_state) => needed_states
                            .entry(light_name.clone())
                            .or_insert(known_state.clone()),
                        None => &mut State::default(),
                    },
                };

                let new_needed_state = previous_needed_state.apply(change);
                if Some(&new_needed_state) != known_states.get(&light_name) {
                    // Ignore errors because we will retry if the state hasn't changed
                    let _ = mqtt
                        .send_new_state(&light_name, &new_needed_state)
                        .await;
                }
                needed_states.insert(light_name, new_needed_state);
            }
            // Timeout
            Err(_) => {
                let known_states = known_states.read().await;
                for light_name in LIGHTS {
                    match needed_states.get(light_name) {
                        Some(needed_state) => {
                            if Some(needed_state)
                                != known_states.get(light_name)
                            {
                                let _ = mqtt
                                    .send_new_state(
                                        &light_name,
                                        &needed_state,
                                    )
                                    .await;
                            }
                        },
                        None => (),
                    }
                }
            }
        }
    }
}

fn extract_state_update(message: Event) -> Option<(String, State)> {
    match message {
        Event::Incoming(incoming) => match incoming {
            Incoming::ConnAck(_)
            | Incoming::PubAck(_)
            | Incoming::PingResp
            | Incoming::SubAck(_) => None,
            Incoming::Publish(message) => {
                let topic: Vec<_> = message.topic.split('/').collect();
                let name = topic[1].to_string();
                let data = &(*message.payload);

                Some((name, data.try_into().unwrap()))
            }
            other => {
                dbg!(other);
                None
            }
        },
        _ => None,
    }
}
