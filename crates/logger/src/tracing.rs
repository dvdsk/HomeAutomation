use tracing::level_filters::LevelFilter;
use tracing_subscriber::registry::LookupSpan;

/// # WARNING
/// part of the filter syntax is broken (sad)
/// see: https://github.com/tokio-rs/tracing/issues/1181
///
/// ## what does work
/// Filter directives like: `crate::module::submodule=trace,crate::module=info,error`
/// which logs everything in `submodule`, in `module` things are logged at level
/// info, warn or error and for the rest of the crate and all dependencies only
/// errors are logged
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
pub fn setup() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::filter;
    use tracing_subscriber::fmt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{self, layer::SubscriberExt};

    let filter = filter::EnvFilter::builder()
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

    let registry = tracing_subscriber::registry()
        .register_filter()
        .with(filter)
        .with(ErrorLayer::default());
    if libsystemd::logging::connected_to_journal() {
        match tracing_journald::layer() {
            Ok(journal) => {
                registry.with(journal).init();
                tracing::info!("Started logging & tracing to journald");
            }
            Err(err) => {
                registry.with(fmt).init();
                tracing::error!(
                    "Could not log to journald directly. Logging to stderr \
                    as fallback. Error connecting to journald:: {err}"
                );
            }
        };
    } else {
        registry.with(fmt).init();
        tracing::info!("Started logging & tracing to stderr");
    }
}
