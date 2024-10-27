#![allow(unused)]
use std::pin::{pin, Pin};

use rumqttc::MqttOptions;
use rumqttc::{AsyncClient, ClientError, EventLoop};
use rumqttc::{ConnectionError, Event};
use serde_json::json;
use tokio::sync::{mpsc, RwLock};

use crate::lights::state::{Change, State};
use crate::{MQTT_IP, MQTT_PORT, QOS};

pub struct CachedBridge {
    mqtt_client: AsyncClient,
    eventloop: EventLoop,
    options: MqttOptions,
}

impl CachedBridge {
    fn new() -> Self {
        let options =
            MqttOptions::new("ha-lightcontroller", MQTT_IP, MQTT_PORT);
        let (mqtt_client, eventloop) = AsyncClient::new(options.clone(), 128);
        Self {
            mqtt_client,
            options,
            eventloop,
        }
    }

    fn send_new_state(&mut self, needed_state: &State) {
        todo!()
    }

    pub async fn set_on(&self, friendly_name: &str) -> Result<(), ClientError> {
        self.set(friendly_name, r#"{"state": "ON"}"#).await
    }

    /// Brightness: 0 to 1
    /// Color temperature: 2200-4000K
    pub async fn set_bri_temp(
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

    pub(crate) async fn set(
        &self,
        friendly_name: &str,
        payload: &str,
    ) -> Result<(), ClientError> {
        let topic = format!("zigbee2mqtt/{friendly_name}/set");

        self.mqtt_client.publish(topic, QOS, false, payload).await?;
        Ok(())
    }

    pub(crate) async fn run(
        mut self,
        mut change_receiver: mpsc::UnboundedReceiver<Change>,
    ) {
        // TODO: init through mqtt get
        let known_state = RwLock::new(State::new());
        let mut needed_state = State::new();
        let options = self.options.clone();
        let mut self_pinned = pin!(self);

        loop {
            let poll_mqtt = self_pinned.as_mut().poll_mqtt(&known_state);

            let handle_changes = self_pinned.as_mut().handle_changes(
                &mut change_receiver,
                &known_state,
                &mut needed_state,
            );

            tokio::select! {
                _ = handle_changes => (),
                _ = poll_mqtt => (),
            }
            println!(
                "Something went wrong with the mqtt connection, reconnecting"
            );
            let (mqtt_client, eventloop) =
                AsyncClient::new(options.clone(), 128);
            self.mqtt_client = mqtt_client;
        }
    }

    async fn handle_changes(
        mut self: Pin<&mut Self>,
        change_receiver: &mut mpsc::UnboundedReceiver<Change>,
        known_state: &RwLock<State>,
        needed_state: &mut State,
    ) -> Result<(), ClientError> {
        loop {
            let change = change_receiver.recv().await;
            *needed_state = needed_state.apply(change);
            let known_state = known_state.read().await;

            if *needed_state != *known_state {
                self.send_new_state(needed_state);
            }
        }
    }

    async fn poll_mqtt(
        mut self: Pin<&mut Self>,
        known_state: &RwLock<State>,
    ) -> Result<(), ConnectionError> {
        loop {
            let message = self.eventloop.poll().await?;
            let new_known_state = extract_state_update(message);

            let mut known_state = known_state.write().await;
            *known_state = new_known_state;
        }
    }
}

// Should return complete new state, otherwise change poll_mqtt
fn extract_state_update(message: Event) -> State {
    todo!()
}

pub(crate) fn mired_to_kelvin(mired: usize) -> usize {
    1_000_000 / mired
}

pub(crate) fn kelvin_to_mired(kelvin: usize) -> usize {
    1_000_000 / kelvin
}
