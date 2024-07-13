#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]

#[cfg(feature = "api")]
pub mod api;
#[cfg(feature = "server")]
pub mod server;
