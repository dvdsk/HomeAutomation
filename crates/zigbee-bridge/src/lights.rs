use rumqttc::{AsyncClient, ClientError, EventLoop, MqttOptions};
use rumqttc::{ConnectionError, Event};
use serde_json::json;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::{mpsc, RwLock};

use super::MQTT_IP;
use super::MQTT_PORT;
use super::QOS;

#[derive(Debug, PartialEq)]
pub(crate) struct State {
    pub(crate) on: bool,
    pub(crate) brightness: f64,
    pub(crate) color_temp: usize,
    pub(crate) color_xy: (f64, f64),
}
impl State {
    fn new() -> Self {
        Self {
            on: todo!(),
            brightness: todo!(),
            color_temp: todo!(),
            color_xy: todo!(),
        }
    }

    fn apply(&mut self, change: Option<StateChange>) -> State {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct StateChange {
    pub(crate) friendly_name: String,
    pub(crate) on: Option<bool>,
    pub(crate) brightness: Option<f64>,
    pub(crate) color_temp: Option<usize>,
    pub(crate) color_xy: Option<(f64, f64)>,
}

#[derive(Clone)]
pub struct Controller {
    pub(crate) change_sender: mpsc::UnboundedSender<StateChange>,
}

pub struct CachedBridge {}

impl CachedBridge {
    pub async fn set_on(
        mqtt_client: AsyncClient,
        friendly_name: &str,
    ) -> Result<(), ClientError> {
        CachedBridge::set(mqtt_client, friendly_name, r#"{"state": "ON"}"#)
            .await
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

        CachedBridge::set(mqtt_client, friendly_name, &payload).await
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

    async fn run(mut change_receiver: mpsc::UnboundedReceiver<StateChange>) {
        let options =
            MqttOptions::new("ha-lightcontroller", MQTT_IP, MQTT_PORT);
        // TODO: init through mqtt get
        let known_state = RwLock::new(State::new());
        let mut needed_state = State::new();

        loop {
            let (mqtt_client, eventloop) =
                AsyncClient::new(options.clone(), 128);
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
            println!(
                "Something went wrong with the mqtt connection, reconnecting"
            );
        }
    }
}

async fn handle_changes(
    change_receiver: &mut mpsc::UnboundedReceiver<StateChange>,
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

impl Controller {
    pub fn start_bridge() -> Self {
        let (change_sender, change_receiver) = mpsc::unbounded_channel();

        let run_bridge = CachedBridge::run(change_receiver);
        tokio::task::spawn(run_bridge);

        Self { change_sender }
    }
}

#[derive(Debug, Error)]
enum MqttError {
    #[error("{0}")]
    Connection(#[from] ConnectionError),
    #[error("{0}")]
    Send(#[from] SendError<Event>),
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
