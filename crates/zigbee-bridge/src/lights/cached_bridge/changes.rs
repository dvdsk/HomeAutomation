use std::collections::HashMap;

use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, timeout};

use super::{mqtt::Mqtt, CHANGE_TIMEOUT, WAIT_FOR_INIT_STATES};
use crate::lights::lamp::{Change, LampState, Model};
use crate::LIGHTS;

pub(super) async fn handle(
    change_receiver: &mut mpsc::UnboundedReceiver<(String, Change)>,
    mqtt: &Mqtt,
    known_states: &RwLock<HashMap<String, LampState>>,
    needed_states: &mut HashMap<String, LampState>,
    devices: &RwLock<HashMap<String, Model>>,
) -> ! {
    // Give the initial known states a chance to be fetched
    sleep(WAIT_FOR_INIT_STATES).await;
    loop {
        if let Ok(change) = timeout(CHANGE_TIMEOUT, change_receiver.recv()).await {
            apply_change(change, known_states, needed_states).await;
        }
        send_all(known_states, needed_states, devices, mqtt).await;
    }
}

async fn send_all(
    known_states: &RwLock<HashMap<String, LampState>>,
    needed_states: &mut HashMap<String, LampState>,
    devices: &RwLock<HashMap<String, Model>>,
    mqtt: &Mqtt,
) {
    let known_states = known_states.read().await;
    for light_name in LIGHTS {
        let Some(needed) = needed_states.get(light_name) else {
            continue;
        };

        let known = known_states.get(light_name);
        if Some(needed) != known {
            let devices = devices.read().await;
            let model = devices.get(light_name).expect("Should be registered");
            // Ignore errors because we will retry if the state hasn't changed
            let _ = mqtt.send_new_state(light_name, needed, model).await;
        }
    }
}

async fn apply_change(
    change: Option<(String, Change)>,
    known_states: &RwLock<HashMap<String, LampState>>,
    needed_states: &mut HashMap<String, LampState>,
) {
    let (light_name, change) = change.expect("Channel should never close");
    let known_states = known_states.read().await;

    let needed = match needed_states.get(&light_name) {
        Some(needed_state) => needed_state,
        None => match known_states.get(&light_name) {
            Some(known_state) => needed_states
                .entry(light_name.clone())
                .or_insert(known_state.clone()),
            None => &mut LampState::default(),
        },
    };

    let with_change_applied = needed.apply(change);
    needed_states.insert(light_name, with_change_applied);
}
