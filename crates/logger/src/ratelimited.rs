use governor::clock::DefaultClock;
use governor::middleware::NoOpMiddleware;
use governor::state::InMemoryState;
use governor::state::NotKeyed;
use governor::Quota;
use governor::RateLimiter;
use std::collections::HashMap;
use std::num::NonZero;
use std::sync::Mutex;
use std::time::Duration;
use tracing::field::Visit;
use tracing::Event;
use tracing::Metadata;
use tracing::Subscriber;
use tracing_core::callsite;
use tracing_core::Interest;
use tracing_subscriber::layer::Context;

struct LimitState {
    limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>,
    supressed: usize,
}
impl LimitState {
    fn new(quota: Quota) -> Self {
        Self {
            limiter: RateLimiter::direct(quota),
            supressed: 0,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct LogMsgHash(u64);
struct CallSite {
    state: LimitState,
    msg: HashMap<LogMsgHash, LimitState>,
}

struct Inner {
    pub callsite: HashMap<callsite::Identifier, CallSite>,
    pub global: LimitState,
}

impl Inner {
    fn event_enabled(&mut self, event: &Event<'_>, quoti: &Quoti) -> bool {
        if self.global.limiter.check().is_err() {
            self.global.supressed += 1;
        } else if self.global.supressed > 0 {
            eprintln!(
                "Logs are being ratelimitted, supressed {} messages",
                self.global.supressed
            );
            self.global.supressed = 0;
        }

        let callsite = self
            .callsite
            .entry(event.metadata().callsite().clone())
            .or_insert(CallSite {
                state: LimitState {
                    limiter: RateLimiter::direct(quoti.callsite),
                    supressed: 0,
                },
                msg: HashMap::new(),
            });

        if callsite.state.limiter.check().is_err() {
            callsite.state.supressed += 1;
            return false;
        } else if callsite.state.supressed > 0 {
            eprintln!(
                "Logs from this callsite are ratelimitted, \
                supressed {} messages",
                callsite.state.supressed
            );
            callsite.state.supressed = 0;
        }

        let mut visitor = Visitor {
            hash: LogMsgHash(0), // placeholder
        };
        event.record(&mut visitor);
        let msg =
            callsite
                .msg
                .entry(visitor.hash)
                .or_insert_with(|| LimitState {
                    limiter: RateLimiter::direct(quoti.msg),
                    supressed: 0,
                });

        if msg.limiter.check().is_err() {
            msg.supressed += 1;
            false
        } else if msg.supressed > 0 {
            eprintln!(
                "Logs with this exact message are ratelimitted, \
                supressed {} messages",
                msg.supressed
            );
            msg.supressed = 0;
            true
        } else {
            true
        }
    }
}

struct Quoti {
    global: Quota,
    callsite: Quota,
    msg: Quota,
}

impl Default for Quoti {
    fn default() -> Self {
        Self {
            global: Quota::per_second(NonZero::new(9).unwrap())
                .allow_burst(NonZero::new(9).unwrap()),
            callsite: Quota::per_second(NonZero::new(6).unwrap())
                .allow_burst(NonZero::new(6).unwrap()),
            msg: Quota::per_second(NonZero::new(1).unwrap())
                .allow_burst(NonZero::new(1).unwrap()),
        }
    }
}

pub struct Limiter {
    inner: Mutex<Inner>,
    quoti: Quoti,
}

impl<S: Subscriber> tracing_subscriber::layer::Filter<S> for Limiter {
    /// Returns `true` if this layer is interested in a span or event with the
    /// given [`Metadata`] in the current [`Context`], similarly to
    /// [`Subscriber::enabled`].
    ///
    /// If this returns `false`, the span or event will be disabled _for the
    /// wrapped [`Layer`]_. Unlike [`Layer::enabled`], the span or event will
    /// still be recorded if any _other_ layers choose to enable it. However,
    /// the layer [filtered] by this filter will skip recording that span or
    /// event.
    ///
    /// If all layers indicate that they do not wish to see this span or event,
    /// it will be disabled.
    ///
    /// [`metadata`]: tracing_core::Metadata
    /// [`Subscriber::enabled`]: tracing_core::Subscriber::enabled
    /// [filtered]: crate::filter::Filtered
    fn enabled(&self, _: &Metadata<'_>, _: &Context<'_, S>) -> bool {
        // all filtering is done based on event_enabled
        true
    }

    fn event_enabled(&self, event: &Event<'_>, _: &Context<'_, S>) -> bool {
        self.inner.lock().unwrap().event_enabled(event, &self.quoti)
    }

    fn callsite_enabled(&self, _: &'static Metadata<'static>) -> Interest {
        Interest::sometimes()
    }
}

struct Visitor {
    hash: LogMsgHash,
}

impl Visit for Visitor {
    fn record_debug(
        &mut self,
        _field: &tracing_core::Field,
        value: &dyn std::fmt::Debug,
    ) {
        self.hash = debug_hash(&value)
    }
}

// adapted from tracing-subscriber/src/filter/env.field.rs
// by Tokio Contributors
fn debug_hash(debug: &impl std::fmt::Debug) -> LogMsgHash {
    // Naively, we would probably hash a value's `fmt::Debug` output by
    // formatting it to a string, and then calculating the stings hash This
    // would however require allocating every time we want to hash a field
    // value
    //
    // Instead, we implement `fmt::Write` for a type that, rather than
    // actually _writing_ the strings to something, hashes them.
    use std::fmt;
    use std::fmt::Write;
    use std::hash::DefaultHasher;
    use std::hash::Hasher;

    struct FmtHasher {
        hash_state: DefaultHasher,
    }

    impl fmt::Write for FmtHasher {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            for byte in s.bytes() {
                self.hash_state.write_u8(byte);
            }
            Ok(())
        }
    }
    let mut hasher = FmtHasher {
        hash_state: DefaultHasher::new(),
    };

    // Try to "write" the value's `fmt::Debug` output to a `Matcher`. This
    // returns an error if the `fmt::Debug` implementation wrote any
    // characters that did not match the expected pattern.
    write!(hasher, "{:?}", debug).expect("hashing can not go wrong");
    LogMsgHash(hasher.hash_state.finish())
}

impl Default for Limiter {
    fn default() -> Self {
        Self {
            inner: Mutex::new(Inner {
                callsite: HashMap::new(),
                global: LimitState::new(Quoti::default().global),
            }),
            quoti: Quoti::default(),
        }
    }
}

impl Limiter {
    pub fn with_global_period(mut self, one_log_per: Duration) -> Self {
        self.quoti.global = Quota::with_period(one_log_per)
            .expect("duration may not be zero")
            .allow_burst(self.quoti.global.burst_size());

        {
            let mut inner = self.inner.lock().unwrap();
            inner.global.limiter = RateLimiter::direct(self.quoti.global)
        }

        self
    }
    pub fn with_callsite_period(
        mut self,
        one_callsite_log_per: Duration,
    ) -> Self {
        self.quoti.callsite = Quota::with_period(one_callsite_log_per)
            .expect("duration may not be zero")
            .allow_burst(self.quoti.callsite.burst_size());

        {
            let mut inner = self.inner.lock().unwrap();
            for CallSite { state, .. } in inner.callsite.values_mut() {
                state.limiter = RateLimiter::direct(self.quoti.callsite)
            }
        }

        self
    }
    pub fn with_msg_period(mut self, one_msg_per: Duration) -> Self {
        self.quoti.msg = Quota::with_period(one_msg_per)
            .expect("duration may not be zero")
            .allow_burst(self.quoti.msg.burst_size());

        {
            let mut inner = self.inner.lock().unwrap();
            for msg_state in inner
                .callsite
                .values_mut()
                .flat_map(|CallSite { msg, .. }| msg.values_mut())
            {
                msg_state.limiter = RateLimiter::direct(self.quoti.msg)
            }
        }

        self
    }

    pub fn with_global_burst(mut self, burst_size: u32) -> Self {
        self.quoti.global =
            Quota::with_period(self.quoti.global.replenish_interval())
                .expect("duration may not be zero")
                .allow_burst(NonZero::new(burst_size).unwrap());

        {
            let mut inner = self.inner.lock().unwrap();
            inner.global.limiter = RateLimiter::direct(self.quoti.global)
        }

        self
    }
    pub fn with_callsite_burst(mut self, burst_size: u32) -> Self {
        self.quoti.callsite =
            Quota::with_period(self.quoti.callsite.replenish_interval())
                .expect("duration may not be zero")
                .allow_burst(NonZero::new(burst_size).unwrap());

        {
            let mut inner = self.inner.lock().unwrap();
            for CallSite { state, .. } in inner.callsite.values_mut() {
                state.limiter = RateLimiter::direct(self.quoti.callsite)
            }
        }

        self
    }
    pub fn with_msg_burst(mut self, burst_size: u32) -> Self {
        self.quoti.callsite =
            Quota::with_period(self.quoti.msg.replenish_interval())
                .expect("duration may not be zero")
                .allow_burst(NonZero::new(burst_size).unwrap());

        {
            let mut inner = self.inner.lock().unwrap();
            for msg_state in inner
                .callsite
                .values_mut()
                .flat_map(|CallSite { msg, .. }| msg.values_mut())
            {
                msg_state.limiter = RateLimiter::direct(self.quoti.msg)
            }
        }

        self
    }
}
