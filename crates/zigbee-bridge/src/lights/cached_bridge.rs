#![allow(unused)]
use std::collections::HashMap;

use rumqttc::MqttOptions;
use rumqttc::{AsyncClient, ClientError, EventLoop};
use rumqttc::{ConnectionError, Event};
use serde_json::json;
use tokio::sync::{mpsc, RwLock};

use crate::lights::conversion::kelvin_to_mired;
use crate::lights::state::{Change, State};
use crate::{LIGHTS, MQTT_IP, MQTT_PORT, QOS};

struct Mqtt {
    client: AsyncClient,
}

impl Mqtt {
    fn send_new_state(&self, light_name: &str, needed_state: &State) {
        todo!()
    }

    async fn set_on(&self, friendly_name: &str) -> Result<(), ClientError> {
        self.set(friendly_name, r#"{"state": "ON"}"#).await
    }

    /// Brightness: 0 to 1
    /// Color temperature: 2200-4000K
    async fn set_bri_temp(
        &self,
        friendly_name: &str,
        brightness: f64,
        color_temp: usize,
    ) -> Result<(), ClientError> {
        let brightness = (brightness * 254.) as usize;
        let payload = json!({
            "brightness": brightness,
            "color_temp": kelvin_to_mired(color_temp)
        })
        .to_string();

        self.set(friendly_name, &payload).await
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

    fn request_state(&self, name: &str) {
        let payload = json!({
            "state": "",
            "brightness": "",
            "color_temp": "",
            "color_xy": "",
        })
        .to_string();
        self.get(name, &payload);
    }
}

pub(crate) async fn run(
    mut change_receiver: mpsc::UnboundedReceiver<(String, Change)>,
) {
    let options = MqttOptions::new("ha-lightcontroller", MQTT_IP, MQTT_PORT);
    // TODO: init through mqtt get
    let known_states = RwLock::new(HashMap::new());
    let mut needed_states = HashMap::new();

    loop {
        let (client, eventloop) = AsyncClient::new(options.clone(), 128);
        let mqtt = Mqtt { client };
        LIGHTS.into_iter().map(|name| mqtt.request_state(name));

        let poll_mqtt = poll_mqtt(eventloop, &known_states);
        let handle_changes = handle_changes(
            &mut change_receiver,
            &mqtt,
            &known_states,
            &mut needed_states,
        );

        tokio::select! {
            _ = handle_changes => (),
            _ = poll_mqtt => (),
        }
        println!("Something went wrong with the mqtt connection, reconnecting");
    }
}

async fn poll_mqtt(
    mut eventloop: EventLoop,
    known_states: &RwLock<HashMap<String, State>>,
) -> Result<(), ConnectionError> {
    loop {
        let message = eventloop.poll().await?;
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
) -> Result<(), ClientError> {
    loop {
        // TODO: add timeout
        let (light_name, change) = change_receiver
            .recv()
            .await
            .expect("Channel should never close");
        let known_states = known_states.read().await;

        let previous_needed_state = match known_states.get(&light_name) {
            Some(known_state) => needed_states
                .entry(light_name.clone())
                .or_insert(known_state.clone()),
            None => &mut State::default(),
        };
        let new_needed_state = previous_needed_state.apply(change);

        if Some(&new_needed_state) != known_states.get(&light_name) {
            mqtt.send_new_state(&light_name, &new_needed_state);
        }
    }
}

// Should return complete new state, otherwise change poll_mqtt
fn extract_state_update(message: Event) -> Option<(String, State)> {
    todo!()
}
