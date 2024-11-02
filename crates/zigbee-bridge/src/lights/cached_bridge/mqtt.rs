use rumqttc::{AsyncClient, ClientError};
use serde_json::json;

use crate::lights::state::Lamp;
use crate::QOS;

pub(super) struct Mqtt {
    client: AsyncClient,
}

impl Mqtt {
    pub(super) fn new(client: AsyncClient) -> Self {
        Mqtt { client }
    }

    pub(super) async fn subscribe(
        &self,
        topic: &str,
    ) -> Result<(), ClientError> {
        self.client.subscribe(topic, QOS).await
    }

    pub(super) async fn request_state(&self, name: &str) {
        println!("Requesting state for light {name}");
        let payload = json!({"state": ""});

        self.get(name, &payload.to_string()).await.unwrap();
    }

    pub(super) async fn send_new_state(
        &self,
        light_name: &str,
        needed_state: &Lamp,
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
}
