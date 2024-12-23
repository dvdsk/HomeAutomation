use zigbee_bridge::Controller;

#[tokio::main]
async fn main() {
    logger::tracing::setup_for_tests();

    let controller = Controller::start_bridge(
        "192.168.1.43".parse().unwrap(),
        "test change radiator temp",
    );
    let radiator = "small_bedroom:radiator";

    controller.set_radiator_setpoint(radiator, 22.);

    let () = std::future::pending().await;
    unreachable!();
}
