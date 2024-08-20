use governor::clock::DefaultClock;
use governor::middleware::NoOpMiddleware;
use governor::state::InMemoryState;
use governor::state::NotKeyed;
use governor::Quota;
use governor::RateLimiter;
use std::num::NonZeroU32;
use std::time::Duration;
use tracing::info;
use tracing::warn;

pub struct RateLimitedLogger {
    pub(crate) rate_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>,
    pub(crate) withheld: usize,
}

impl RateLimitedLogger {
    pub fn new() -> Self {
        let quota = Quota::with_period(Duration::from_secs(1))
            .unwrap()
            .allow_burst(NonZeroU32::new(5).unwrap());
        Self {
            rate_limiter: RateLimiter::direct(quota),
            withheld: 0,
        }
    }

    pub fn warn(&mut self, txt: &str) {
        if self.rate_limiter.check().is_err() {
            self.withheld += 1;
            return;
        }

        if self.withheld > 0 {
            warn!("Logs are being spammed, withheld {} logs", self.withheld);
            self.withheld = 0;
        }

        warn!(txt);
    }

    pub fn info(&mut self, txt: &str) {
        if self.rate_limiter.check().is_err() {
            self.withheld += 1;
            return;
        }

        if self.withheld > 0 {
            info!("Logs are being spammed, withheld {} logs", self.withheld);
            self.withheld = 0;
        }

        info!(txt);
    }
}
