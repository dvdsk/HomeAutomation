// use logger::{ratelimited::Logger, warn};
//
// fn main() {
//     logger::tracing::setup();
//
//     let mut logger = Logger::default().with_per_msg_period(Duration::from_millis(800));
//
//     for i in 0..10 {
//         warn!(logger; "constant message always the same");
//         warn!(logger; "oh no the number is: {}", i);
//         thread::sleep(Duration::from_millis(100));
//     }
// }

use std::time::Duration;

fn log_a() {
    tracing::info!("a");
}

fn log_b() {
    tracing::info!("b");
}

fn log_c() {
    tracing::info!("c");
}


fn main() {
    logger::tracing::setup();

    // functions otherwise the callsite will be different
    log_a();
    log_a();
    log_a();
    log_b();
    log_a();
    log_b();
    log_a();
    log_c();
    log_a();

    std::thread::sleep(Duration::from_secs(2));
    log_a();
    log_b();
    log_c();
}
