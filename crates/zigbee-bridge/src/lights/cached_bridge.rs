use std::collections::HashMap;

use rumqttc::v5::{AsyncClient, MqttOptions};
use tokio::sync::{mpsc, RwLock};
use tracing::trace;

use self::mqtt::Mqtt;
use crate::lights::lamp::Change;
use crate::{LIGHTS, MQTT_IP, MQTT_PORT};

mod changes;
mod mqtt;
mod poll;

pub(super) async fn run(
    change_receiver: mpsc::UnboundedReceiver<(String, Change)>,
) -> ! {
    let mut options =
        MqttOptions::new("ha-lightcontroller", MQTT_IP, MQTT_PORT);
    // Set max mqtt packet size to 4kB
    options.set_max_packet_size(Some(4096));
    // Keep subscriptions when reconnecting!!!!
    options.set_clean_start(false);

    let known_states = RwLock::new(HashMap::new());

    // Reconnecting to broker is handled by Eventloop::poll
    let channel_capacity = 128;
    let (client, eventloop) =
        AsyncClient::new(options.clone(), channel_capacity);
    let mut mqtt = Mqtt::new(client);

    mqtt.subscribe("zigbee2mqtt/bridge/devices").await.unwrap();
    for light in LIGHTS {
        mqtt.subscribe(&format!("zigbee2mqtt/{light}"))
            .await
            .unwrap();
        mqtt.request_state(light).await;
    }

    trace!("Starting main zigbee management loops");
    let poll_mqtt = poll::poll_mqtt(eventloop, &known_states);
    let handle_changes =
        changes::handle(change_receiver, &mut mqtt, &known_states);

    tokio::select! {
        () = handle_changes => unreachable!("should not panic"),
        () = poll_mqtt => unreachable!("should not panic")
    }
}