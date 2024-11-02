// hue_power_on_behavior: "recover" / power_on_behavior: "previous"
// state: ON/OFF
// brightness: 0-254
// color_temp: 153-500 (mired) / 250-454
// color_temp_startup
// color_xy
// Set to on, off after 30s
// on_time: 30, off_wait_time: 30

use rumqttc::QoS;

pub mod lights;

const QOS: QoS = QoS::AtLeastOnce;
const MQTT_IP: &str = "192.168.1.43";
const MQTT_PORT: u16 = 1883;
// TODO: get from bridge
const LIGHTS: [&str; 2] = ["kitchen:fridge", "kitchen:hallway"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lights::Controller;
    use std::time::Duration;

    #[tokio::test]
    async fn start_bridge() {
        let controller = Controller::start_bridge();

        for light in LIGHTS {
            controller.set_brightness(light, 0.5);
            controller.set_color_temp(light, 2200);
        }

        tokio::time::sleep(Duration::from_secs(2)).await;

        for light in LIGHTS {
            controller.set_off(light);
        }

        tokio::time::sleep(Duration::from_secs(2)).await;

        for light in LIGHTS {
            controller.set_on(light);
            controller.set_color_temp(light, 4000);
        }

        tokio::time::sleep(Duration::from_secs(2)).await;

        for light in LIGHTS {
            controller.set_brightness(light, 1.0);
        }

        let () = std::future::pending().await;
        unreachable!();
    }
}
