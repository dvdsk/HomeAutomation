use governor::clock::DefaultClock;
use governor::middleware::NoOpMiddleware;
use governor::state::InMemoryState;
use governor::state::NotKeyed;
use governor::Quota;
use governor::RateLimiter;
use std::collections::HashMap;
use std::collections::HashSet;
use std::num::NonZero;
use std::num::NonZeroU32;
use std::sync::Mutex;
use std::time::Duration;
use tracing::field::Visit;
use tracing::Metadata;
use tracing::Subscriber;
use tracing_core::callsite;
use tracing_core::span;
use tracing_core::Interest;
use tracing_subscriber::layer::Context;

struct Message {
    limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>,
    supressed: usize,
}

struct LogMsgHash(u64);
struct CallSite {
    limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>,
    supressed: usize,
    msg_limited: HashMap<LogMsgHash, Message>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct Key {
    callsite: callsite::Identifier,
}

impl Key {
    fn from_meta(meta: &Metadata<'_>) -> Key {
        Key {
            callsite: meta.callsite(),
        }
    }
}

struct Inner {
    pub callsite: HashMap<callsite::Identifier, CallSite>,
    pub callsite_by_span_id: HashMap<span::Id, callsite::Identifier>,
    pub global: CallSite,
}

fn span_quota() -> Quota {
    Quota::per_second(NonZero::new(1).unwrap()).allow_burst(NonZero::new(10).unwrap())
}

fn message_quota() -> Quota {
    Quota::per_second(NonZero::new(1).unwrap()).allow_burst(NonZero::new(10).unwrap())
}

fn global_quota() -> Quota {
    Quota::per_second(NonZero::new(1).unwrap()).allow_burst(NonZero::new(10).unwrap())
}

impl Inner {
    /// Should this be filtered? Determined by the work of on_record
    fn enabled(&mut self, meta: &Metadata<'_>) -> bool {
    }

    fn on_record(&mut self, span_id: &span::Id, values: &span::Record<'_>) {
        let callsite_id = self
            .callsite_by_span_id
            .get(span_id)
            .expect("on_new_span handler should run before on_record");
        let callsite = self
            .callsite
            .entry(callsite_id.clone())
            .or_insert(CallSite {
                limiter: RateLimiter::direct(span_quota()),
                withheld: 0,
                msg_limited: HashMap::new(),
            });

        if callsite.limiter.check().is_err() {
            callsite.supressed += 1;
            return
        }

        let mut visitor = Visitor {
            hash: LogMsgHash(0), // placeholder
        };
        values.record(&mut visitor);
    }

    pub fn on_new_span(&mut self, attrs: &span::Attributes<'_>, span_id: &span::Id) {
        self.callsite_by_span_id
            .insert(span_id.clone(), attrs.metadata().callsite());
    }
}

pub struct Limiter {
    inner: Mutex<Inner>,
    // pub msg_rate_limiter:
    //     RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock, NoOpMiddleware>,
    // pub global_rate_limiter:
    // pub was_withheld: HashSet<String>,
    // pub withheld: usize,
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
    fn enabled(&self, meta: &Metadata<'_>, _: &Context<'_, S>) -> bool {
        self.inner.lock().unwrap().enabled(meta)
    }

    /// uses visitor pattern (sorry, tracing uses it so we must ...)
    /// derived from tracing-subscriber EnvFilter::on_record
    /// -> span.record_update
    /// -> record.record
    /// -> m.visitor()
    /// -> impl<'a> Visit for MatchVisitor
    fn on_record(&self, id: &span::Id, values: &span::Record<'_>, _: Context<'_, S>) {
        self.inner.lock().unwrap().on_record(id, values)
    }

    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, _: Context<'_, S>) {
        self.inner.lock().unwrap().on_new_span(attrs, id);
    }

    fn callsite_enabled(&self, _: &'static Metadata<'static>) -> Interest {
        Interest::sometimes()
    }
}

struct Visitor {
    hash: LogMsgHash,
}

impl Visit for Visitor {
    fn record_debug(&mut self, _field: &tracing_core::Field, value: &dyn std::fmt::Debug) {
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
    let mut hasher = FmtHasher { hash_state: DefaultHasher::new() };

    // Try to "write" the value's `fmt::Debug` output to a `Matcher`. This
    // returns an error if the `fmt::Debug` implementation wrote any
    // characters that did not match the expected pattern.
    write!(hasher, "{:?}", debug).is_ok();
    LogMsgHash(hasher.hash_state.finish())
}

impl Default for Limiter {
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
            was_withheld: HashSet::<String>,
        }
    }
}

impl Limiter {
    pub fn with_per_msg_period(self, one_msg_per: Duration) -> Self {
        let quota = Quota::with_period(one_msg_per)
            .unwrap()
            .allow_burst(NonZeroU32::new(5).unwrap());
        Self {
            msg_rate_limiter: RateLimiter::keyed(quota),
            global_rate_limiter: self.global_rate_limiter,
            withheld: self.withheld,
            was_withheld: self.was_withheld,
        }
    }
}
