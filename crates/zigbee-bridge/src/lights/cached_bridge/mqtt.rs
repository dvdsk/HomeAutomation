use std::collections::HashMap;
use std::time::Instant;

use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{AsyncClient, ClientError};
use serde_json::json;
use tracing::{trace, warn};

use crate::lights::lamp;

use super::TIME_IT_TAKES_TO_APPLY_CHANGE;

pub(super) struct Mqtt {
    client: AsyncClient,
    // TODO: extract into SendTracker struct?
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

    pub(super) fn next_deadline(
        &self,
        light_name: &str,
        diff: &[lamp::Property],
    ) -> Option<Instant> {
        diff.into_iter()
            .map(|change| self.change_next_due(light_name, &change))
            .min()
    }

    fn change_next_due(
        &self,
        light_name: &str,
        change: &lamp::Property,
    ) -> Instant {
        let Some(light_send_record) = self.last_sent.get(light_name) else {
            // lamp has never been sent before
            return Instant::now();
        };

        let Some((sent_at, prev_change)) =
            light_send_record.get(&change.into())
        else {
            // property has never been sent before
            return Instant::now();
        };

        // there is a new change
        if prev_change != change {
            return Instant::now();
        }

        // we are overdue
        if sent_at.elapsed() > TIME_IT_TAKES_TO_APPLY_CHANGE {
            return Instant::now();
        }

        *sent_at + TIME_IT_TAKES_TO_APPLY_CHANGE
    }

    fn is_due(&self, light_name: &str, change: &lamp::Property) -> bool {
        let deadline = self.change_next_due(light_name, change);
        deadline < Instant::now()
    }

    pub(super) async fn send_diff_where_due(
        &mut self,
        light_name: &str,
        merged_payloads: bool,
        diff: &[lamp::Property],
    ) -> Result<(), ClientError> {
        let mut due_changes = Vec::new();

        for change in diff {
            if self.is_due(&light_name, change) {
                due_changes.push(change);

                let light_send_record =
                    self.last_sent.entry(light_name.to_owned()).or_default();
                light_send_record
                    .insert(change.into(), (Instant::now(), *change));
            }
        }

        if !due_changes.is_empty() {
            if merged_payloads {
                let payload = merge_payloads(due_changes);
                self.set(&light_name, payload.to_string()).await?;
            } else {
                for change in due_changes {
                    self.set(&light_name, change.payload().to_string()).await?;
                }
            }
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

        if topic.contains("small_bedroom:bureau") {
            warn!("ZB to MQTT (bureau): {payload}");
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

fn merge_payloads(mut changes: Vec<&lamp::Property>) -> serde_json::Value {
    let payload = changes
        .iter_mut()
        .map(|c| c.payload())
        .map(|p| p.as_object().expect("Should be a map").to_owned())
        .reduce(|mut acc, mut e| {
            acc.append(&mut e);
            acc
        })
        .unwrap();
    serde_json::Value::Object(payload.clone())
}
