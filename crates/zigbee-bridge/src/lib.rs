// hue_power_on_behavior: "recover" / power_on_behavior: "previous"
// color_temp_startup
// color_xy
// Set to on, off after 30s
// on_time: 30, off_wait_time: 30

use rumqttc::v5::mqttbytes::QoS;

pub mod lights;

const QOS: QoS = QoS::AtMostOnce;
const MQTT_IP: &str = "192.168.1.43";
const MQTT_PORT: u16 = 1883;
const LIGHTS: [&str; 5] = [
    "kitchen:fridge",
    "kitchen:hallway",
    "kitchen:hood_left",
    "kitchen:hood_right",
    "kitchen:ceiling",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lights::Controller;
    use std::time::Duration;

    #[ignore]
    #[tokio::test]
    async fn change_all_lights() {
        std::env::set_var("RUST_LOG", "brain=trace,zigbee_bridge=trace,info"); 
        let controller = Controller::start_bridge();

        println!("Setting to on, 2200");
        for light in LIGHTS {
            controller.set_on(light);
            controller.set_color_temp(light, 2200);
        }

        tokio::time::sleep(Duration::from_secs(1)).await;

        println!("Turning off");
        for light in LIGHTS {
            controller.set_off(light);
        }

        tokio::time::sleep(Duration::from_secs(1)).await;

        println!("Turning on to 4000");
        for light in LIGHTS {
            controller.set_on(light);
            controller.set_color_temp(light, 4000);
        }

        tokio::time::sleep(Duration::from_secs(1)).await;

        println!("Setting bri to 1.0");
        for light in LIGHTS {
            controller.set_brightness(light, 1.0);
        }

        let () = std::future::pending().await;
        unreachable!();
    }
}
