use std::time::Duration;

use serde::{Deserialize, Serialize};

pub mod client;
pub use client::reconnecting::Client as ReconnectingClient;
pub use client::reconnecting::SubscribedClient as ReconnectingSubscribedClient;
pub use client::Client;
pub use client::Subscribed as SubscribedClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Handshake { name: String },
    Actuate(protocol::Affector),
    Subscribe,
    ListAffectors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Handshake,
    Error(ServerError),
    Actuate(Result<(), AffectorError>),
    ListAffectors(Vec<protocol::Affector>),
    SubUpdate(SubMessage),
    Subscribe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubMessage {
    Reading(protocol::Reading),
    ErrorReport(Box<protocol::Error>),
}

#[derive(Clone, Debug, thiserror::Error, Serialize, Deserialize)]
pub enum AffectorError {
    #[error("We do not have a connection to the actuator's node")]
    Offline,
}

#[derive(Clone, Debug, thiserror::Error, Serialize, Deserialize)]
#[error("placeholder")]
pub struct SubscribeError;

#[derive(Clone, Debug, thiserror::Error, Serialize, Deserialize)]
#[error("placeholder")]
pub enum ServerError {
    TooManyRequests(Duration),
}
