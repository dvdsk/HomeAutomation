use ratelimited_logger::{warn, RateLimitedLogger};

fn main() {
    setup_tracing();
    let mut logger = RateLimitedLogger::new();

    warn!(logger; "oh no the number is: {}", 5);
}

fn setup_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::filter;
use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{self, layer::SubscriberExt, Layer};

    let subscriber = tracing_subscriber::fmt::layer()
        .with_line_number(true)
        .with_target(true)
        .with_filter(filter::LevelFilter::INFO);
    tracing_subscriber::registry()
        .with(subscriber)
        .with(ErrorLayer::default())
        .init();
}
