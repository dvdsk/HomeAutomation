use governor::clock::DefaultClock;
use governor::middleware::NoOpMiddleware;
use governor::state::InMemoryState;
use governor::state::NotKeyed;
use governor::Quota;
use governor::RateLimiter;
use std::num::NonZeroU32;
use std::time::Duration;

pub struct RateLimitedLogger {
    pub rate_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>,
    pub withheld: usize,
}

#[macro_export]
macro_rules! warn {
    ($limiter:ident; $($arg:tt)*) => {
        if $limiter.rate_limiter.check().is_err() {
            $limiter.withheld += 1;
        } else {
            if $limiter.withheld > 0 {
                tracing::warn!("Logs are being spammed, withheld {} logs", $limiter.withheld);
                $limiter.withheld = 0;
            }

            tracing::warn!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! info {
    ($limiter:ident; $($arg:tt)*) => {
        if $limiter.rate_limiter.check().is_err() {
            $limiter.withheld += 1;
        } else {
            if $limiter.withheld > 0 {
                tracing::info!("Logs are being spammed, withheld {} logs", $limiter.withheld);
                $limiter.withheld = 0;
            }

            tracing::info!($($arg)*);
        }
    };
}

impl Default for RateLimitedLogger {
    fn default() -> Self {
        let quota = Quota::with_period(Duration::from_secs(1))
            .unwrap()
            .allow_burst(NonZeroU32::new(5).unwrap());
        Self {
            rate_limiter: RateLimiter::direct(quota),
            withheld: 0,
        }
    }
}

impl RateLimitedLogger {
    pub fn warn(&mut self, txt: &str) {
        if self.rate_limiter.check().is_err() {
            self.withheld += 1;
            return;
        }

        if self.withheld > 0 {
            tracing::warn!("Logs are being spammed, withheld {} logs", self.withheld);
            self.withheld = 0;
        }

        tracing::warn!(txt);
    }

    pub fn info(&mut self, txt: &str) {
        if self.rate_limiter.check().is_err() {
            self.withheld += 1;
            return;
        }

        if self.withheld > 0 {
            tracing::info!("Logs are being spammed, withheld {} logs", self.withheld);
            self.withheld = 0;
        }

        tracing::info!(txt);
    }
}
