use std::{net::IpAddr, time::Duration};

use rumqttc::v5::{AsyncClient, MqttOptions};
use tokio::sync::{mpsc, RwLock};
use tracing::trace;

use self::mqtt::Mqtt;
use crate::{device::init_states, lamp::LampProperty, LIGHT_MODELS, MQTT_PORT};

mod changes;
mod mqtt;
mod poll;

const MQTT_MIGHT_BE_DOWN_TIMEOUT: Duration = Duration::from_secs(500);
const WAIT_FOR_INIT_STATES: Duration = Duration::from_millis(500);
// unfortunately we have to use the zigbee-herdsman timeout to prevent
// the z2m queue from getting filled up
const TIME_IT_TAKES_TO_APPLY_CHANGE: Duration = Duration::from_secs(10);
const CHANGE_ACCUMULATION_TIME: Duration = Duration::from_millis(100);

pub(super) async fn run(
    mqtt_ip: IpAddr,
    change_receiver: mpsc::UnboundedReceiver<(String, LampProperty)>,
) -> ! {
    let mut options =
        MqttOptions::new("ha-lightcontroller", mqtt_ip.to_string(), MQTT_PORT);
    // Set max mqtt packet size to 4kB
    options.set_max_packet_size(Some(4096));
    // Keep subscriptions when reconnecting!!!!
    options.set_clean_start(false);

    let known_states = RwLock::new(init_states());

    // Reconnecting to broker is handled by Eventloop::poll
    let channel_capacity = 128;
    let (client, eventloop) =
        AsyncClient::new(options.clone(), channel_capacity);
    let mut mqtt = Mqtt::new(client);

    mqtt.subscribe("zigbee2mqtt/bridge/logging").await.unwrap();
    mqtt.subscribe("zigbee2mqtt/bridge/event").await.unwrap();
    for (light, _) in LIGHT_MODELS {
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
