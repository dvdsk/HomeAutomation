use std::{collections::HashMap, time::Duration};

use rumqttc::v5::{AsyncClient, MqttOptions};
use tokio::sync::{mpsc, RwLock};
use tracing::trace;

use self::mqtt::Mqtt;
use crate::{lights::lamp, MQTT_IP, MQTT_PORT};

mod changes;
mod mqtt;
mod poll;

const MQTT_MIGHT_BE_DOWN_TIMEOUT: Duration = Duration::from_secs(500);
const WAIT_FOR_INIT_STATES: Duration = Duration::from_millis(500);
const TIME_IT_TAKES_TO_APPLY_CHANGE: Duration = Duration::from_secs(1);

const TO_SUBSCRIBE: [&str; 2] =
    ["zigbee2mqtt/bridge/devices", "zigbee2mqtt/bridge/logging"];

pub(super) async fn run(
    change_receiver: mpsc::UnboundedReceiver<(String, lamp::Property)>,
) -> ! {
    let mut options =
        MqttOptions::new("ha-lightcontroller", MQTT_IP, MQTT_PORT);
    // Set max mqtt packet size to 4kB
    options.set_max_packet_size(Some(4096));
    // We need a clean session to receive the device list...
    // Now we need to resubscribe when we reconnect...
    options.set_clean_start(true);

    let known_states = RwLock::new(HashMap::new());

    // Reconnecting to broker is handled by Eventloop::poll
    let channel_capacity = 128;
    let (client, eventloop) =
        AsyncClient::new(options.clone(), channel_capacity);
    let mqtt = RwLock::new(Mqtt::new(client));

    trace!("Starting main zigbee management loops");
    let poll_mqtt = poll::poll_mqtt(eventloop, &mqtt, &known_states);
    let handle_changes = changes::handle(change_receiver, &mqtt, &known_states);

    tokio::select! {
        () = handle_changes => unreachable!("should not panic"),
        () = poll_mqtt => unreachable!("should not panic")
    }
}
