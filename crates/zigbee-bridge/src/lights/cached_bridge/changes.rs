use std::collections::HashMap;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;
use tracing::{debug, instrument, trace};

use super::mqtt::Mqtt;
use crate::lights::lamp::{self, Lamp};
use crate::LIGHTS;

pub(super) async fn handle(
    mut change_receiver: mpsc::UnboundedReceiver<(String, lamp::Property)>,
    mqtt: &mut Mqtt,
    known_states: &RwLock<HashMap<String, Lamp>>,
) -> ! {
    const MQTT_MIGHT_BE_DOWN_TIMEOUT: Duration = Duration::from_secs(500);
    const WAIT_FOR_INIT_STATES: Duration = Duration::from_millis(500);

    // Give the initial known states a chance to be fetched
    sleep(WAIT_FOR_INIT_STATES).await;

    let mut needed_states = HashMap::new();
    let mut timeout = MQTT_MIGHT_BE_DOWN_TIMEOUT;

    loop {
        debug!("timeout: {timeout:?}");
        if let Ok(update) =
            tokio::time::timeout(timeout, change_receiver.recv()).await
        {
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
        };

        timeout = on_change_or_timeout(known_states, &mut needed_states, mqtt)
            .await
            .min(MQTT_MIGHT_BE_DOWN_TIMEOUT);
    }
}

/// Might not be done in case a light property in needed does not match known
/// however has recently been set/send. Needs a recheck in the near future to
/// make sure the set/send takes effect. We do not send it again now as that
/// would be a little spammy. Returns when we need to recheck. If we do not need
/// to do so we return Duration::MAX.
#[instrument(skip_all)]
async fn on_change_or_timeout(
    known_states: &RwLock<HashMap<String, Lamp>>,
    needed_states: &mut HashMap<String, Lamp>,
    mqtt: &mut Mqtt,
) -> Duration {
    let mut new_timeout = Duration::MAX;
    let known_states = known_states.read().await;

    for light_name in LIGHTS {
        tracing::Span::current().record("light_name", light_name);

        let Some(needed) = needed_states.get(light_name) else {
            continue;
        };

        let diff = match known_states.get(light_name) {
            Some(known) => needed.changes_relative_to(known),
            None => needed.all_as_changes(),
        };

        if !diff.is_empty() {
            let _ = mqtt.send_diff(light_name, &diff).await;

            let this_light_due = mqtt.earliest_change_due(light_name, &diff);
            let this_light_timeout =
                this_light_due.saturating_duration_since(Instant::now());
            new_timeout = new_timeout.min(this_light_timeout);
        }
    }
    new_timeout
}

#[instrument(skip_all)]
async fn apply_change_to_needed(
    light_name: String,
    change: lamp::Property,
    known_states: &RwLock<HashMap<String, Lamp>>,
    needed_states: &mut HashMap<String, Lamp>,
) {
    let known_states = known_states.read().await;

    let needed = match needed_states.get(&light_name) {
        Some(needed_state) => needed_state,
        None => match known_states.get(&light_name) {
            Some(known_state) => needed_states
                .entry(light_name.clone())
                .or_insert(known_state.clone()),
            None => &mut Lamp::default(),
        },
    };

    let with_change_applied = needed.clone().apply(change);
    needed_states.insert(light_name, with_change_applied);
}
