use std::collections::HashMap;

use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, timeout};
use tracing::{instrument, trace};

use super::{mqtt::Mqtt, CHANGE_TIMEOUT, WAIT_FOR_INIT_STATES};
use crate::lights::lamp::{Change, Lamp};
use crate::LIGHTS;

pub(super) async fn handle(
    change_receiver: &mut mpsc::UnboundedReceiver<(String, Change)>,
    mqtt: &Mqtt,
    known_states: &RwLock<HashMap<String, Lamp>>,
    needed_states: &mut HashMap<String, Lamp>,
) -> ! {
    // Give the initial known states a chance to be fetched
    sleep(WAIT_FOR_INIT_STATES).await;
    loop {
        if let Ok(change) = timeout(CHANGE_TIMEOUT, change_receiver.recv()).await {
            trace!("Received change: {change:?}");
            apply_change(change, known_states, needed_states).await;
        }
        send_all(known_states, needed_states, mqtt).await;
    }
}

#[instrument(skip_all)]
async fn send_all(
    known_states: &RwLock<HashMap<String, Lamp>>,
    needed_states: &mut HashMap<String, Lamp>,
    mqtt: &Mqtt,
) {
    let known_states = known_states.read().await;
    for light_name in LIGHTS {
        let Some(needed) = needed_states.get(light_name) else {
            continue;
        };

        let Some(known) = known_states.get(light_name) else {
            // Ignore errors because we will retry if the state hasn't changed
            let _ = mqtt.send_new_state(light_name, needed).await;
            continue;
        };

        if needed != known {
            if light_name == "kitchen:ceiling" {
                trace!(
                    "Lamp {light_name}
                sending needed {needed:?} 
                to replace known {known:?}"
                );
            }
            // Ignore errors because we will retry if the state hasn't changed
            let _ = mqtt.send_new_state(light_name, needed).await;
        }
    }
}

#[instrument(skip_all)]
async fn apply_change(
    change: Option<(String, Change)>,
    known_states: &RwLock<HashMap<String, Lamp>>,
    needed_states: &mut HashMap<String, Lamp>,
) {
    let (light_name, change) = change.expect("Channel should never close");
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
