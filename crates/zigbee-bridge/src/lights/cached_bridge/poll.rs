use std::collections::HashMap;

use rumqttc::{ConnectionError, Event, EventLoop, Incoming};
use tokio::sync::RwLock;

use crate::lights::state::State;

pub(super) async fn poll_mqtt(
    mut eventloop: EventLoop,
    known_states: &RwLock<HashMap<String, State>>,
) -> Result<(), ConnectionError> {
    loop {
        let message = match eventloop.poll().await {
            Ok(message) => message,
            Err(err) => {
                println!("Error while polling: {err}");
                continue;
            }
        };

        if let Some((light_name, new_known_state)) =
            extract_state_update(message)
        {
            let mut known_states = known_states.write().await;
            known_states.insert(light_name, new_known_state);
        }
    }
}

fn extract_state_update(message: Event) -> Option<(String, State)> {
    match message {
        Event::Incoming(incoming) => match incoming {
            Incoming::ConnAck(_)
            | Incoming::PubAck(_)
            | Incoming::PingResp
            | Incoming::SubAck(_) => None,
            Incoming::Publish(message) => {
                let topic: Vec<_> = message.topic.split('/').collect();
                let name = topic[1].to_string();
                let data = &(*message.payload);

                Some((name, data.try_into().unwrap()))
            }
            other => {
                dbg!(other);
                None
            }
        },
        Event::Outgoing(_) => None,
    }
}
