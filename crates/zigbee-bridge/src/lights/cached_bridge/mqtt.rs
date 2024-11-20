use std::collections::HashMap;
use std::time::{Duration, Instant};

use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{AsyncClient, ClientError};
use serde_json::json;
use tracing::{instrument, trace, warn};

use crate::lights::lamp;

pub(super) struct Mqtt {
    client: AsyncClient,
    last_sent: HashMap<
        String,
        HashMap<lamp::PropertyDiscriminants, (Instant, lamp::Property)>,
    >,
}

impl Mqtt {
    pub(super) fn new(client: AsyncClient) -> Self {
        Mqtt {
            client,
            last_sent: HashMap::new(),
        }
    }

    pub(super) async fn subscribe(
        &self,
        topic: &str,
    ) -> Result<(), ClientError> {
        // Its okay for messages to arrive twice or more. MQTT guarantees
        // ordering and we only do something if the cached bridge indicates we
        // need to so light states arriving twice is not an issue.
        self.client.subscribe(topic, QoS::AtLeastOnce).await
    }

    pub(super) async fn request_state(&self, name: &str) {
        trace!("Requesting state for light {name}");
        let payload = json!({"state": ""});

        self.get(name, payload.to_string()).await.unwrap();
    }

    /// Pre-condition: diff non-empty
    pub(super) fn earliest_change_due(
        &self,
        light_name: &str,
        diff: &[lamp::Property],
    ) -> Instant {
        diff.into_iter()
            .map(|change| self.change_next_due(light_name, &change))
            .min()
            .expect("Diff should be non-empty")
    }

    fn change_next_due(
        &self,
        light_name: &str,
        change: &lamp::Property,
    ) -> Instant {
        const CHANGE_APPLY_DELAY: Duration = Duration::from_secs(1);

        let Some(light_send_record) = self.last_sent.get(light_name) else {
            // lamp has never been sent before
            return Instant::now();
        };

        let Some(prop_send_record) = light_send_record.get(&change.into())
        else {
            // property has never been sent before
            return Instant::now();
        };

        let (sent_at, prev_change) = prop_send_record;

        if prev_change != change || sent_at.elapsed() > CHANGE_APPLY_DELAY {
            // we are overdue
            Instant::now()
        } else {
            *sent_at + CHANGE_APPLY_DELAY
        }
    }

    #[instrument(skip(self))]
    pub(super) async fn send_diff(
        &mut self,
        light_name: &str,
        diff: &[lamp::Property],
    ) -> Result<(), ClientError> {
        for change in diff {
            self.set(&light_name, change.payload()).await?;

            let light_send_record =
                self.last_sent.entry(light_name.to_owned()).or_default();
            light_send_record.insert(change.into(), (Instant::now(), *change));
        }

        Ok(())
    }

    async fn set(
        &self,
        friendly_name: &str,
        payload: String,
    ) -> Result<(), ClientError> {
        let topic = format!("zigbee2mqtt/{friendly_name}/set");

        trace!("Sending payload {payload} to lamp {friendly_name}");
        self.publish(&topic, payload).await?;
        Ok(())
    }

    async fn get(
        &self,
        friendly_name: &str,
        payload: String,
    ) -> Result<(), ClientError> {
        let topic = format!("zigbee2mqtt/{friendly_name}/get");

        self.publish(&topic, payload).await?;
        Ok(())
    }

    async fn publish(
        &self,
        topic: &str,
        payload: String,
    ) -> Result<(), ClientError> {
        let properties = rumqttc::v5::mqttbytes::v5::PublishProperties {
            message_expiry_interval: Some(5), // seconds
            ..Default::default()
        };

        if topic.contains("kitchen:fridge") {
            warn!("ZB to MQTT (fridge): {payload}");
        }
        self.client
            .publish_with_properties(
                topic,
                QoS::AtLeastOnce,
                false,
                payload,
                properties,
            )
            .await
    }
}
