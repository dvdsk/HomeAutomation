use std::collections::HashMap;
use std::time::Duration;

use tokio::sync::{mpsc, RwLock};

use super::{mqtt::Mqtt, CHANGE_TIMEOUT};
use crate::lights::state::{Change, State};
use crate::LIGHTS;

pub(super) async fn handle(
    change_receiver: &mut mpsc::UnboundedReceiver<(String, Change)>,
    mqtt: &Mqtt,
    known_states: &RwLock<HashMap<String, State>>,
    needed_states: &mut HashMap<String, State>,
) -> ! {
    // Give the initial known states a chance to be fetched
    tokio::time::sleep(Duration::from_millis(500)).await;
    loop {
        if let Ok(change) =
            tokio::time::timeout(CHANGE_TIMEOUT, change_receiver.recv()).await
        {
            apply_change(change, known_states, needed_states).await;
        }
        send_all(known_states, needed_states, mqtt).await;
    }
}

async fn send_all(
    known_states: &RwLock<HashMap<String, State>>,
    needed_states: &mut HashMap<String, State>,
    mqtt: &Mqtt,
) {
    let known_states = known_states.read().await;
    for light_name in LIGHTS {
        if let Some(needed_state) = needed_states.get(light_name) {
            if Some(needed_state) != known_states.get(light_name) {
                // Ignore errors because we will retry if the state hasn't changed
                let _ = mqtt.send_new_state(light_name, needed_state).await;
            }
        }
    }
}

async fn apply_change(
    change: Option<(String, Change)>,
    known_states: &RwLock<HashMap<String, State>>,
    needed_states: &mut HashMap<String, State>,
) {
    let (light_name, change) = change.expect("Channel should never close");
    let known_states = known_states.read().await;

    let previous_needed_state = match needed_states.get(&light_name) {
        Some(needed_state) => needed_state,
        None => match known_states.get(&light_name) {
            Some(known_state) => needed_states
                .entry(light_name.clone())
                .or_insert(known_state.clone()),
            None => &mut State::default(),
        },
    };

    let new_needed_state = previous_needed_state.apply(change);

    needed_states.insert(light_name, new_needed_state);
}
