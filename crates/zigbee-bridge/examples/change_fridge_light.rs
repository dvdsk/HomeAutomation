use zigbee_bridge::Controller;
use std::time::Duration;

#[tokio::main]
async fn main() {
    logger::tracing::setup_for_tests();

    let controller = Controller::start_bridge(
        "192.168.1.43".parse().unwrap(),
        "test change fridge light",
    );
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
