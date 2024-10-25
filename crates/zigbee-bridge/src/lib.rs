// hue_power_on_behavior: "recover" / power_on_behavior: "previous"
// state: ON/OFF
// brightness: 0-254
// color_temp: 153-500 (mired) / 250-454
// color_temp_startup
// color_xy
// Set to on, off after 30s
// on_time: 30, off_wait_time: 30

use rumqttc::{
    AsyncClient, ClientError, ConnectionError, Event, EventLoop, MqttOptions,
    QoS,
};
use serde_json::json;

const QOS: QoS = QoS::AtLeastOnce;

pub struct LightController {
    client: AsyncClient,
    eventloop: EventLoop,
}

impl LightController {
    pub fn new() -> Self {
        let options =
            MqttOptions::new("ha-lampcontroller", "192.168.1.43", 1883);
        let (client, eventloop) = AsyncClient::new(options, 128);

        Self { client, eventloop }
    }

    pub async fn poll(&mut self) -> Result<Event, ConnectionError> {
        self.eventloop.poll().await
    }

    pub async fn set_on(&self, friendly_name: &str) -> Result<(), ClientError> {
        self.set(friendly_name, r#"{"state": "ON"}"#).await
    }

    pub async fn set_off(
        &self,
        friendly_name: &str,
    ) -> Result<(), ClientError> {
        self.set(friendly_name, r#"{"state": "OFF"}"#).await
    }

    /// Brightness: 0 to 1
    /// Color temperature: 2200-4000K
    pub async fn set_bri_temp(
        &self,
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

        self.set(friendly_name, &payload).await
    }

    async fn set(
        &self,
        friendly_name: &str,
        payload: &str,
    ) -> Result<(), ClientError> {
        let topic = format!("zigbee2mqtt/{friendly_name}/set");

        self.client
            .publish(topic, QOS, false, payload)
            .await?;
        Ok(())
    }
}

fn mired_to_kelvin(mired: usize) -> usize {
    1_000_000 / mired
}

fn kelvin_to_mired(kelvin: usize) -> usize {
    1_000_000 / kelvin
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn set_hallway_off() {
        let mut controller = LightController::new();
        controller.set_off("gangkast tafellamp").await.unwrap();
        loop {
            dbg!(controller.poll().await.unwrap());
        }
    }

    #[tokio::test]
    async fn set_hallway_dim() {
        let mut controller = LightController::new();
        controller.set_bri_temp("gangkast tafellamp", 0.1, 2200).await.unwrap();
        loop {
            dbg!(controller.poll().await.unwrap());
        }
    }
}
