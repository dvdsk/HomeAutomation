use std::collections::HashMap;
use std::time::{Duration, Instant};

use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{AsyncClient, ClientError};
use serde_json::json;
use tracing::{instrument, trace};

use crate::lights::lamp::{LampProperty, LampPropertyDiscriminants};

pub(super) struct Mqtt {
    client: AsyncClient,
    property_last_set: HashMap<String, HashMap<LampPropertyDiscriminants, (Instant, LampProperty)>>,
}

impl Mqtt {
    pub(super) fn new(client: AsyncClient) -> Self {
        Mqtt {
            client,
            property_last_set: HashMap::new(),
        }
    }

    pub(super) async fn subscribe(&self, topic: &str) -> Result<(), ClientError> {
        // Its okay for messages to arrive twice or more. MQTT guarantees
        // ordering and we only do something if the cached bridge indicates we
        // need to so light states arriving twice is not an issue.
        self.client.subscribe(topic, QoS::AtLeastOnce).await
    }

    pub(super) async fn request_state(&self, name: &str) {
        trace!("Requesting state for light {name}");
        let payload = json!({"state": ""});

        get(&self.client, name, payload.to_string()).await.unwrap();
    }

    #[instrument(skip(self))]
    pub(super) async fn try_send_state_diff(
        &mut self,
        light_name: String,
        diff: Vec<LampProperty>,
    ) -> Result<Duration, ClientError> {
        const TIME_IT_TAKES_TO_APPLY_CHANGE: Duration = Duration::from_secs(1);
        let mut new_call_needed_in: Duration = Duration::MAX;

        let last_set = self
            .property_last_set
            .entry(light_name.clone())
            .or_default();

        for change in diff {
            let Some((at, prev_change)) = last_set.get(&change.into()) else {
                set(&self.client, &light_name, change.payload()).await?;
                last_set.insert(change.into(), (Instant::now(), change));
                continue;
            };

            if *prev_change != change {
                set(&self.client, &light_name, change.payload()).await?;
                last_set.insert(change.into(), (Instant::now(), change));
                continue;
            }

            if at.elapsed() > TIME_IT_TAKES_TO_APPLY_CHANGE {
                set(&self.client, &light_name, change.payload()).await?;
                last_set.insert(change.into(), (Instant::now(), change));
                continue;
            }

            // trace!(
            //     "not setting property {change:?} for {light_name} as it has \
            //     recently been set"
            // );
            let next_call_allowed = *at + TIME_IT_TAKES_TO_APPLY_CHANGE;
            let until = next_call_allowed.saturating_duration_since(Instant::now());
            new_call_needed_in = new_call_needed_in.min(until);
        }

        Ok(new_call_needed_in)
    }
}

async fn set(
    client: &AsyncClient,
    friendly_name: &str,
    payload: String,
) -> Result<(), ClientError> {
    let topic = format!("zigbee2mqtt/{friendly_name}/set");

    trace!("Sending payload {payload} to lamp {friendly_name}");
    publish(client, &topic, payload).await?;
    Ok(())
}

async fn get(
    client: &AsyncClient,
    friendly_name: &str,
    payload: String,
) -> Result<(), ClientError> {
    let topic = format!("zigbee2mqtt/{friendly_name}/get");

    publish(client, &topic, payload).await?;
    Ok(())
}

async fn publish(client: &AsyncClient, topic: &str, payload: String) -> Result<(), ClientError> {
    let properties = rumqttc::v5::mqttbytes::v5::PublishProperties {
        message_expiry_interval: Some(5), // seconds
        ..Default::default()
    };

    client
        .publish_with_properties(topic, QoS::AtLeastOnce, false, payload, properties)
        .await
}
