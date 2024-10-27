use rumqttc::MqttOptions;
use rumqttc::{AsyncClient, ClientError, EventLoop};
use rumqttc::{ConnectionError, Event};
use serde_json::json;
use tokio::sync::{mpsc, RwLock};

use crate::lights::state::{Change, State};
use crate::{MQTT_IP, MQTT_PORT, QOS};

pub async fn set_on(
    mqtt_client: AsyncClient,
    friendly_name: &str,
) -> Result<(), ClientError> {
    set(mqtt_client, friendly_name, r#"{"state": "ON"}"#).await
}

/// Brightness: 0 to 1
/// Color temperature: 2200-4000K
pub async fn set_bri_temp(
    mqtt_client: AsyncClient,
    friendly_name: &str,
    brightness: f64,
    color_temp: usize,
) -> Result<(), ClientError> {
    let brightness = (brightness * 254.) as usize;
    let payload = json!({
        "brightness": brightness,
        "color_temp": kelvin_to_mired(color_temp)
    })
    .to_string();

    set(mqtt_client, friendly_name, &payload).await
}

pub(crate) async fn set(
    mqtt_client: AsyncClient,
    friendly_name: &str,
    payload: &str,
) -> Result<(), ClientError> {
    let topic = format!("zigbee2mqtt/{friendly_name}/set");

    mqtt_client.publish(topic, QOS, false, payload).await?;
    Ok(())
}

pub(crate) async fn run(mut change_receiver: mpsc::UnboundedReceiver<Change>) {
    let options = MqttOptions::new("ha-lightcontroller", MQTT_IP, MQTT_PORT);
    // TODO: init through mqtt get
    let known_state = RwLock::new(State::new());
    let mut needed_state = State::new();

    loop {
        let (mqtt_client, eventloop) = AsyncClient::new(options.clone(), 128);
        let poll_mqtt = poll_mqtt(eventloop, &known_state);

        let handle_changes = handle_changes(
            &mut change_receiver,
            mqtt_client,
            &known_state,
            &mut needed_state,
        );

        tokio::select! {
            _ = handle_changes => (),
            _ = poll_mqtt => (),
        }
        println!("Something went wrong with the mqtt connection, reconnecting");
    }
}

async fn poll_mqtt(
    mut eventloop: EventLoop,
    known_state: &RwLock<State>,
) -> Result<(), ConnectionError> {
    loop {
        let message = eventloop.poll().await?;
        let new_known_state = extract_state_update(message);

        let mut known_state = known_state.write().await;
        *known_state = new_known_state;
    }
}

async fn handle_changes(
    change_receiver: &mut mpsc::UnboundedReceiver<Change>,
    mut mqtt_client: AsyncClient,
    known_state: &RwLock<State>,
    needed_state: &mut State,
) -> Result<(), ClientError> {
    loop {
        let change = change_receiver.recv().await;
        *needed_state = needed_state.apply(change);
        let known_state = known_state.read().await;

        if *needed_state != *known_state {
            send_new_state(&mut mqtt_client, needed_state);
        }
    }
}

fn send_new_state(mqtt_client: &mut AsyncClient, needed_state: &State) {
    todo!()
}

// Should return complete new state, otherwise change poll_mqtt
fn extract_state_update(message: Event) -> State {
    todo!()
}

pub(crate) fn mired_to_kelvin(mired: usize) -> usize {
    1_000_000 / mired
}

pub(crate) fn kelvin_to_mired(kelvin: usize) -> usize {
    1_000_000 / kelvin
}
