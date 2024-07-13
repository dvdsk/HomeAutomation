#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

#[cfg(feature = "api")]
pub mod subscriber;
#[cfg(feature = "api")]
pub use subscriber::{Subscriber, AsyncSubscriber, SubMessage};

#[cfg(feature = "server")]
pub mod server;
