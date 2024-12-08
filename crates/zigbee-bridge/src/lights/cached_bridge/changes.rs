use std::collections::HashMap;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, timeout};
use tracing::{debug, instrument, trace};

use super::mqtt::Mqtt;
use super::{
    CHANGE_ACCUMULATION_TIME, MQTT_MIGHT_BE_DOWN_TIMEOUT, WAIT_FOR_INIT_STATES,
};
use crate::lights::lamp::{self, Lamp};

pub(super) async fn handle(
    mut change_receiver: mpsc::UnboundedReceiver<(String, lamp::Property)>,
    mqtt: &mut Mqtt,
    known_states: &RwLock<HashMap<String, Lamp>>,
) -> ! {
    // Give the initial known states a chance to be fetched
    sleep(WAIT_FOR_INIT_STATES).await;

    let mut needed_states = HashMap::new();
    let mut call_at_least_in = MQTT_MIGHT_BE_DOWN_TIMEOUT;

    loop {
        debug!("timeout: {call_at_least_in:?}");
        match timeout(call_at_least_in, change_receiver.recv()).await {
            // On change, update needed, but only actually send the changes
            // after a timeout
            Ok(update) => {
                let (light_name, change) =
                    update.expect("Channel should never close");

                trace!("Received change: {change:?} for lamp {light_name}");
                apply_change_to_needed(
                    light_name,
                    change,
                    known_states,
                    &mut needed_states,
                )
                .await;

                // When there hasn't been a new change in 100 ms, we will timeout
                // and send the accumulated changes
                call_at_least_in = CHANGE_ACCUMULATION_TIME;
            }
            _ => {
                // Send the accumulated changes and get the timeout for resending
                call_at_least_in = send_diff_get_timeout(
                    known_states,
                    &mut needed_states,
                    mqtt,
                )
                .await
                .min(MQTT_MIGHT_BE_DOWN_TIMEOUT);
            }
        };
    }
}

/// Might not be done in case a light property in needed does not match known
/// however has recently been set/send. Needs a recheck in the near future to
/// make sure the set/send takes effect. We do not send it again now as that
/// would be a little spammy. Returns when we need to recheck. If we do not need
/// to do so we return Duration::MAX.
#[instrument(skip_all)]
async fn send_diff_get_timeout(
    known_states: &RwLock<HashMap<String, Lamp>>,
    needed_states: &mut HashMap<String, Lamp>,
    mqtt: &mut Mqtt,
) -> Duration {
    let known_states = known_states.read().await;
    let mut light_deadlines = Vec::new();

    for (light_name, needed) in needed_states {
        tracing::Span::current().record("light_name", light_name);

        let diff = match known_states.get(light_name) {
            Some(known) => needed.changes_relative_to(known),
            None => needed.all_as_changes(),
        };

        let is_online = match known_states.get(light_name) {
            Some(known) => known.is_online,
            // we assume the lamp is online so that init messages get sent
            None => true,
        };

        if is_online {
            let merged_payloads = needed.is_hue_lamp();
            let _ = mqtt
                .send_diff_where_due(light_name, merged_payloads, &diff)
                .await;
        }

        if let Some(deadline) = mqtt.next_deadline(light_name, &diff) {
            light_deadlines.push(deadline);
        }
    }

    light_deadlines
        .into_iter()
        .min()
        .map(|deadline| deadline.saturating_duration_since(Instant::now()))
        .unwrap_or(Duration::MAX)
}

#[instrument(skip_all)]
async fn apply_change_to_needed(
    light_name: String,
    change: lamp::Property,
    known_states: &RwLock<HashMap<String, Lamp>>,
    needed_states: &mut HashMap<String, Lamp>,
) {
    let known_states = known_states.read().await;

    let known = known_states
        .get(&light_name)
        .map(|l| l.to_owned())
        .unwrap_or_else(|| Lamp::new(&light_name));
    let mut needed = match needed_states.get(&light_name) {
        Some(needed) => needed.clone(),
        None => known,
    };

    needed.apply(change);
    needed_states.insert(light_name, needed);
}
