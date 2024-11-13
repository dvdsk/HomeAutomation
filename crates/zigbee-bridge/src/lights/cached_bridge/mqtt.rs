use std::collections::HashMap;
use std::time::{Duration, Instant};

use rumqttc::v5::{AsyncClient, ClientError};
use serde_json::json;
use tracing::{instrument, trace, warn};

use crate::lights::lamp::{LampProperty, LampPropertyDiscriminants};
use crate::QOS;

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
        self.client.subscribe(topic, QOS).await
    }

    pub(super) async fn request_state(&self, name: &str) {
        trace!("Requesting state for light {name}");
        let payload = json!({"state": ""});

        get(&self.client, name, payload.to_string()).await.unwrap();
    }

    #[instrument(skip(self))]
    pub(super) async fn try_send_state_diff(
        &mut self,
        light_name: &str,
        diff: Vec<LampProperty>,
    ) -> Result<Duration, ClientError> {
        const TIME_IT_TAKES_TO_APPLY_CHANGE: Duration = Duration::from_secs(1);
        let mut new_call_needed_in: Duration = Duration::MAX;

        if let Some(last_set) = self.property_last_set.get_mut(light_name) {
            for change in diff {
                match last_set.get(&change.into()) {
                    None => {
                        set(&self.client, light_name, change.payload()).await?;
                        last_set.insert(change.into(), (Instant::now(), change));
                    }
                    Some((at, prev_change))
                        if *prev_change == change
                            && at.elapsed() > TIME_IT_TAKES_TO_APPLY_CHANGE =>
                    {
                        set(&self.client, light_name, change.payload()).await?;
                        last_set.insert(change.into(), (Instant::now(), change));
                    }
                    Some((at, _)) => {
                        let until = at.saturating_duration_since(Instant::now());
                        new_call_needed_in = new_call_needed_in.min(until);
                    }
                }
            }
        } else {
            let mut last_set = HashMap::new();
            for change in diff {
                let change_key: LampPropertyDiscriminants = change.into();
                set(&self.client, light_name, change.payload()).await?;
                last_set.insert(change_key, Instant::now());
            }
        }

        Ok(new_call_needed_in)
    }
}

async fn set(client: &AsyncClient, friendly_name: &str, payload: String) -> Result<(), ClientError> {
    let topic = format!("zigbee2mqtt/{friendly_name}/set");

    trace!("Sending payload {payload} to lamp {friendly_name}");
    if friendly_name == "kitchen:hallway" && payload.contains("state") {
        warn!("Sending payload {payload} to {friendly_name}");
    }
    publish(client, &topic, payload).await?;
    Ok(())
}

async fn get(client: &AsyncClient, friendly_name: &str, payload: String) -> Result<(), ClientError> {
    let topic = format!("zigbee2mqtt/{friendly_name}/get");

    publish(client, &topic, payload).await?;
    Ok(())
}

async fn publish(client: &AsyncClient, topic: &str, payload: String) -> Result<(), ClientError> {
    client.publish(topic, QOS, false, payload).await
}
