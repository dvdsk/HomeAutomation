use std::collections::HashMap;
use std::time::Duration;

use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, timeout};
use tracing::{instrument, trace};

use super::mqtt::Mqtt;
use crate::lights::lamp::{Change, Lamp};
use crate::LIGHTS;

pub(super) async fn handle(
    mut change_receiver: mpsc::UnboundedReceiver<(String, Change)>,
    mqtt: &mut Mqtt,
    known_states: &RwLock<HashMap<String, Lamp>>,
) -> ! {
    const MQTT_MIGHT_BE_DOWN_TIMEOUT: Duration = Duration::from_secs(5);
    const WAIT_FOR_INIT_STATES: Duration = Duration::from_millis(500);

    // Give the initial known states a chance to be fetched
    sleep(WAIT_FOR_INIT_STATES).await;
    let mut needed_states = HashMap::new();
    let mut call_again_in = MQTT_MIGHT_BE_DOWN_TIMEOUT;

    loop {
        if let Ok(res) = timeout(call_again_in, change_receiver.recv()).await {
            let res = res.expect("Channel should never close");
            let (light_name, change) = res;

            trace!("Received change: {change:?}");
            apply_change(light_name, change, known_states, &mut needed_states).await;
        };

        call_again_in = send_and_queue(known_states, &mut needed_states, mqtt)
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
async fn send_and_queue(
    known_states: &RwLock<HashMap<String, Lamp>>,
    needed_states: &mut HashMap<String, Lamp>,
    mqtt: &mut Mqtt,
) -> Duration {
    let mut call_again_in = Duration::MAX;
    let known_states = known_states.read().await;

    for light_name in LIGHTS {
        let Some(needed) = needed_states.get(light_name) else {
            continue;
        };

        let Some(known) = known_states.get(light_name) else {
            // Ignore errors because we will retry if the state hasn't changed
            if let Ok(dur) = mqtt
                .try_send_state_diff(light_name, needed.property_list())
                .await
            {
                call_again_in = call_again_in.min(dur);
            }
            continue;
        };

        if needed != known {
            let diff = needed.changes_relative_to(known);
            // Ignore errors because we will retry if the state hasn't changed
            if let Ok(dur) = mqtt.try_send_state_diff(light_name, diff).await {
                call_again_in = call_again_in.min(dur);
            }
        }
    }
    call_again_in
}

#[instrument(skip_all)]
async fn apply_change(
    light_name: String,
    change: Change,
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
