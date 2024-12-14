use crate::Controller;
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

#[tokio::test]
async fn change_radiator_temp() {
    std::env::set_var("RUST_LOG", "brain=trace,zigbee_bridge=trace,info");
    let controller =
        Controller::start_bridge("192.168.1.43".parse().unwrap());
    let radiator = "small_bedroom:radiator";

    controller.set_radiator_setpoint(radiator, 22.);

    let () = std::future::pending().await;
    unreachable!();
}
