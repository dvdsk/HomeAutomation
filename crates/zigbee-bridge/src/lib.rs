// hue_power_on_behavior: "recover" / power_on_behavior: "previous"
// state: ON/OFF
// brightness: 0-254
// color_temp: 153-500 (mired) / 250-454
// color_temp_startup
// color_xy
// Set to on, off after 30s
// on_time: 30, off_wait_time: 30

use rumqttc::QoS;

pub use lights::Controller;

mod lights;

const QOS: QoS = QoS::AtLeastOnce;
const MQTT_IP: &str = "192.168.1.43";
const MQTT_PORT: u16 = 1883;
// TODO: get from bridge
const LIGHTS: [&str; 2] = ["keuken tafellamp", "gangkast tafellamp"];

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::Controller;

    #[tokio::test]
    async fn start_bridge() {
        let controller = Controller::start_bridge();

        controller.set_brightness("gangkast tafellamp", 0.5);
        controller.set_brightness("keuken tafellamp", 0.5);

        controller.set_color_temp("gangkast tafellamp", 2200);
        controller.set_color_temp("keuken tafellamp", 2200);

        tokio::time::sleep(Duration::from_secs(2)).await;

        controller.set_off("gangkast tafellamp");
        controller.set_off("keuken tafellamp");

        tokio::time::sleep(Duration::from_secs(2)).await;

        controller.set_on("gangkast tafellamp");
        controller.set_on("keuken tafellamp");

        controller.set_color_temp("gangkast tafellamp", 4000);
        controller.set_color_temp("keuken tafellamp", 4000);

        tokio::time::sleep(Duration::from_secs(2)).await;

        controller.set_brightness("gangkast tafellamp", 1.0);
        controller.set_brightness("keuken tafellamp", 1.0);

        let () = std::future::pending().await;
        unreachable!();
    }
}
