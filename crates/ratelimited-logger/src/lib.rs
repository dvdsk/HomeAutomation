use governor::clock::DefaultClock;
use governor::middleware::NoOpMiddleware;
use governor::state::keyed::DefaultKeyedStateStore;
use governor::state::InMemoryState;
use governor::state::NotKeyed;
use governor::Quota;
use governor::RateLimiter;
use std::num::NonZeroU32;
use std::time::Duration;

pub struct RateLimitedLogger {
    pub msg_rate_limiter:
        RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock, NoOpMiddleware>,
    pub global_rate_limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>,
    pub withheld: usize,
}

#[macro_export]
macro_rules! warn {
    ($limiter:ident; $($arg:tt)*) => {
        { // scope so msg cannot interfere with locals with same name
            let msg = format!($($arg)*);
            if $limiter.global_rate_limiter.check().is_err() {
                $limiter.withheld += 1;
            } else if $limiter.msg_rate_limiter.check_key(&msg).is_err() {
                $limiter.withheld += 1;
            } else {
                if $limiter.withheld > 0 {
                    tracing::warn!("Logs are being spammed, withheld {} copies of this msg", $limiter.withheld);
                    $limiter.withheld = 0;
                }

                tracing::warn!("{msg}");
            }
        }
    };
}

#[macro_export]
macro_rules! info {
    ($limiter:ident; $($arg:tt)*) => {
        { // scope so msg cannot interfere with locals with same name
            let msg = format!($($arg)*);
            if $limiter.global_rate_limiter.check().is_err() {
                $limiter.withheld += 1;
            } else if $limiter.msg_rate_limiter.check_key(&msg).is_err() {
                $limiter.withheld += 1;
            } else {
                if $limiter.withheld > 0 {
                    tracing::info!("Logs are being spammed, withheld {} copies of this msgs", $limiter.withheld);
                    $limiter.withheld = 0;
                }

                tracing::info!("{msg}");
            }
        }
    };
}

impl Default for RateLimitedLogger {
    fn default() -> Self {
        let per_msg_quota = Quota::with_period(Duration::from_secs(5))
            .unwrap()
            .allow_burst(NonZeroU32::new(5).unwrap());
        let global_quota = Quota::with_period(Duration::from_secs(1))
            .unwrap()
            .allow_burst(NonZeroU32::new(25).unwrap());
        Self {
            msg_rate_limiter: RateLimiter::keyed(per_msg_quota),
            global_rate_limiter: RateLimiter::direct(global_quota),
            withheld: 0,
        }
    }
}

impl RateLimitedLogger {
    pub fn with_per_msg_period(self, one_msg_per: Duration) -> Self {
        let quota = Quota::with_period(one_msg_per)
            .unwrap()
            .allow_burst(NonZeroU32::new(5).unwrap());
        Self {
            msg_rate_limiter: RateLimiter::keyed(quota),
            global_rate_limiter: self.global_rate_limiter,
            withheld: self.withheld,
        }
    }
}
