// hue_power_on_behavior: "recover" / power_on_behavior: "previous"
// color_temp_startup
// color_xy
// Set to on, off after 30s
// on_time: 30, off_wait_time: 30

use lights::Model;

pub mod lights;
mod device;

const MQTT_PORT: u16 = 1883;
const LIGHT_MODELS: [(&str, Model); 16] = [
    ("kitchen:fridge", Model::TradfriE14Color),
    ("kitchen:hallway", Model::TradfriE27),
    ("kitchen:hood_left", Model::TradfriCandle),
    ("kitchen:hood_right", Model::TradfriCandle),
    ("kitchen:ceiling", Model::HueGen4),
    ("large_bedroom:cabinet", Model::TradfriGU10),
    ("large_bedroom:ceiling", Model::TradfriE27),
    ("large_bedroom:desk", Model::TradfriE27),
    ("large_bedroom:wardrobe", Model::TradfriE27),
    ("large_bedroom:bed", Model::TradfriE14White),
    ("small_bedroom:ceiling", Model::HueGen4),
    ("small_bedroom:bureau", Model::HueGen4),
    ("small_bedroom:piano", Model::HueGen4),
    ("bathroom:ceiling", Model::HueGen4),
    ("hallway:ceiling", Model::TradfriE27),
    ("toilet:ceiling", Model::HueGen4),
];

#[cfg(test)]
mod tests {
    use crate::lights::Controller;
    use std::time::Duration;

    #[ignore]
    #[tokio::test]
    async fn change_fridge_light() {
        std::env::set_var("RUST_LOG", "brain=trace,zigbee_bridge=trace,info");
        let controller =
            Controller::start_bridge("192.168.1.43".parse().unwrap());
        let light = "kitchen:fridge";

        println!("Setting to on, 2200");
        controller.set_on(light);
        controller.set_color_temp(light, 2200);

        tokio::time::sleep(Duration::from_secs(1)).await;

        println!("Turning off");
        controller.set_off(light);

        tokio::time::sleep(Duration::from_secs(1)).await;

        println!("Turning on to 4000");
        controller.set_on(light);
        controller.set_color_temp(light, 4000);

        tokio::time::sleep(Duration::from_secs(1)).await;

        println!("Setting bri to 1.0");
        controller.set_brightness(light, 1.0);

        let () = std::future::pending().await;
        unreachable!();
    }
}
