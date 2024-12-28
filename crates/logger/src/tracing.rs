use std::time::Duration;

use crate::ratelimited;

use tracing::level_filters::LevelFilter;
use tracing_error::ErrorLayer;
use tracing_subscriber::filter;
use tracing_subscriber::fmt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
use tracing_subscriber::{self, layer::SubscriberExt};

/// # WARNING
/// part of the filter syntax is broken (sad)
/// see: https://github.com/tokio-rs/tracing/issues/1181
///
/// ## what does work
///
/// ### log everything in module:
/// Filter directives like:
/// `RUST_LOG=crate::module::submodule=trace,crate::module=info,error`
/// which logs everything in `submodule`, in `module` things are logged at level
/// info, warn or error and for the rest of the crate and all dependencies only
/// errors are logged
///
/// ### log everything in function
/// RUST_LOG='[function_name]=trace'
///
/// ### Print if argument matches regex:
/// you can do this with: RUST_LOG='[{topic=.*small_bedroom:piano.*}]=trace,info'
/// that will print every log at trace level or higher that is inside an
/// instrumented function with an argument topic for which the regex
/// .*small_bedroom.* evaluates as true
///
/// ## what should work but only does so sporadically
/// Filter directives allowing you to match field values (values recorded by for
/// example `#[instrument]`),
///
/// their syntax:
/// RUST_LOG=target[span{field=value}]=level
///
/// field value will be interpreted as regular expressions if it cannot be interpreted as
/// bool, i64, u64, or f64 literal. Regex syntax follows the regex crate.
///
/// ### Example:
/// run something like this with: RUST_LOG='[shave{yak=2}]' and it should only
/// print the trace message once
/// ```rust
/// #[tracing::instrument]
/// pub fn shave(yak: usize) {
///     tracing::trace!("I am going to shave a yak :)")
/// }
///
/// fn main() {
///     shave(1);
///     shave(2);
///     shave(3);
/// }
/// ```
///
/// for full docs see: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html
///
/// # Note
/// If the service runs under systemd it logs to the sysjournal. Then:
///
/// The standard journald CODE_LINE and CODE_FILE fields are automatically
/// emitted. A TARGET field is emitted containing the event’s target. For
/// events recorded inside spans, an additional SPAN_NAME field is emitted
/// with the name of each of the event’s parent spans. User-defined fields
/// other than the event message field have a prefix applied by default to
/// prevent collision with standard fields.
///
/// example: `journalctl -fu desk-sensors
/// --output-fields=CODE_FILE,CODE_LINE,MESSAGE -o cat`
pub fn setup() {
    let env_filter = filter::EnvFilter::builder()
        .with_regex(true)
        .try_from_env()
        .unwrap_or_else(|_| {
            filter::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .parse_lossy("")
        });

    let ratelimiter = ratelimited::Limiter::default()
        .with_global_period(Duration::from_secs(1))
        .with_global_burst(20)
        .with_callsite_period(Duration::from_secs(5))
        .with_global_burst(10)
        .with_msg_period(Duration::from_secs(20))
        .with_msg_burst(10);

    let fmt = fmt::layer()
        .pretty()
        .with_writer(std::io::stderr) // to stderr as to not disrupt TUI's
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_ansi(true);

    let registry =
        tracing_subscriber::Registry::default().with(ErrorLayer::default());

    if libsystemd::logging::connected_to_journal() {
        match tracing_journald::layer() {
            Ok(journal) => {
                registry
                    .with(
                        journal
                            .with_filter(ratelimiter)
                            .with_filter(env_filter),
                    )
                    .init();
                tracing::info!("Started logging & tracing to journald");
            }
            Err(err) => {
                registry
                    .with(fmt.with_filter(ratelimiter).with_filter(env_filter))
                    .init();
                tracing::error!(
                    "Could not log to journald directly. Logging to stderr \
                    as fallback. Error connecting to journald:: {err}"
                );
            }
        };
    } else {
        registry
            .with(fmt.with_filter(ratelimiter).with_filter(env_filter))
            .init();
        tracing::info!("Started logging & tracing to stderr");
    }
}

pub fn setup_unlimited() {
    let env_filter = filter::EnvFilter::builder()
        .with_regex(true)
        .try_from_env()
        .unwrap_or_else(|_| {
            filter::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .parse_lossy("")
        });

    let fmt = fmt::layer()
        .pretty()
        .with_writer(std::io::stderr) // to stderr as to not disrupt TUI's
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_ansi(true);

    tracing_subscriber::Registry::default()
        .with(ErrorLayer::default())
        .with(fmt.with_filter(env_filter))
        .init();
}

pub fn setup_for_tests() {
    use std::sync::Once;
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{
        self, layer::SubscriberExt, util::SubscriberInitExt, Layer,
    };

    static INIT: Once = Once::new();

    INIT.call_once(|| {
        color_eyre::install().unwrap();

        let file_subscriber = tracing_subscriber::fmt::layer()
            .with_test_writer()
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .with_ansi(true)
            .pretty()
            .with_filter(
                tracing_subscriber::filter::EnvFilter::from_default_env(),
            );
        tracing_subscriber::registry()
            .with(file_subscriber)
            .with(ErrorLayer::default())
            .init();
    })
}
