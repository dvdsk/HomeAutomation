use std::collections::HashMap;
use std::time::Duration;

use rumqttc::{AsyncClient, ClientError, EventLoop};
use rumqttc::{ConnectionError, Event};
use rumqttc::{Incoming, MqttOptions};
use serde_json::json;
use tokio::sync::{mpsc, RwLock};

use crate::lights::state::{Change, State};
use crate::{LIGHTS, MQTT_IP, MQTT_PORT, QOS};

mod changes;

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
    let handle_changes = changes::handle(
        &mut change_receiver,
        &mqtt,
        &known_states,
        &mut needed_states,
    );

    tokio::select! {
        () = handle_changes => (),
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
        Event::Outgoing(_) => None,
    }
}
