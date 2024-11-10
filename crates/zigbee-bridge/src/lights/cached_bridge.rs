use std::collections::HashMap;
use std::time::Duration;

use rumqttc::{AsyncClient, MqttOptions};
use tokio::sync::{mpsc, RwLock};
use tracing::trace;

use self::mqtt::Mqtt;
use crate::lights::lamp::Change;
use crate::{LIGHTS, MQTT_IP, MQTT_PORT};

mod changes;
mod mqtt;
mod poll;

const CHANGE_TIMEOUT: Duration = Duration::from_secs(5);
const WAIT_FOR_INIT_STATES: Duration = Duration::from_millis(500);

pub(super) async fn run(
    mut change_receiver: mpsc::UnboundedReceiver<(String, Change)>,
) -> ! {
    let mut options =
        MqttOptions::new("ha-lightcontroller", MQTT_IP, MQTT_PORT);
    // Set incoming to max mqtt packet size, outgoing to rumqtt default
    options.set_max_packet_size(2_usize.pow(28), 10240);

    let known_states = RwLock::new(HashMap::new());
    let mut needed_states = HashMap::new();

    // Reconnecting to broker is handled by Eventloop::poll
    let (client, eventloop) = AsyncClient::new(options.clone(), 128);
    let mqtt = Mqtt::new(client);

    mqtt.subscribe("zigbee2mqtt/bridge/devices").await.unwrap();
    for light in LIGHTS {
        mqtt.subscribe(&format!("zigbee2mqtt/{light}"))
            .await
            .unwrap();
        mqtt.request_state(light).await;
    }

    trace!("Starting main zigbee management loops");
    let poll_mqtt = poll::poll_mqtt(eventloop, &known_states);
    let handle_changes = changes::handle(
        &mut change_receiver,
        &mqtt,
        &known_states,
        &mut needed_states,
    );

    tokio::select! {
        () = handle_changes => unreachable!("should not panic"),
        () = poll_mqtt => unreachable!("should not panic")
    }
}
