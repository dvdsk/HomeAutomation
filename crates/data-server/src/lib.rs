#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

#[cfg(feature = "api")]
pub mod api;

#[cfg(feature = "server")]
pub mod server;
