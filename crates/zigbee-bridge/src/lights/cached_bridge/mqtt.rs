use rumqttc::{AsyncClient, ClientError};
use serde_json::json;
use tracing::{trace, warn};

use crate::lights::lamp::Lamp;
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
        trace!("Requesting state for light {name}");
        let payload = json!({"state": ""});

        self.get(name, &payload.to_string()).await.unwrap();
    }

    pub(super) async fn send_new_state(
        &self,
        light_name: &str,
        needed: &Lamp,
    ) -> Result<(), ClientError> {
        // TODO: Can we not send all the json for each lamp in one go? I
        // remember you told me something about this but I forgot what it was.
        // Might be useful to have a line documenting that here. <11-11-24,
        // dvdsk>
        for payload in needed.to_payloads() {
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

        trace!("Sending payload {payload} to lamp {friendly_name}");
        if friendly_name == "kitchen:hallway" && payload.contains("state") {
            warn!("Sending payload {payload} to {friendly_name}");
        }
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
