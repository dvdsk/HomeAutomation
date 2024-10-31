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
    use super::Controller;

    #[tokio::test]
    async fn start_bridge() {
        let _controller = Controller::start_bridge().await;

        let () = std::future::pending().await;
        unreachable!();
    }
}
