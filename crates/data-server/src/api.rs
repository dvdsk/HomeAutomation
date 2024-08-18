use serde::{Deserialize, Serialize};

pub mod client;
pub use client::reconnecting::Client as ReconnectingClient;
pub use client::reconnecting::SubscribedClient as ReconnectingSubscribedClient;
pub use client::{Client, SubscribedClient};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Handshake { name: String },
    Actuate(protocol::Affector),
    Subscribe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Handshake,
    Error(ServerError),
    Actuate,
    SubUpdate(SubMessage),
    Subscribe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubMessage {
    Reading(protocol::Reading),
    ErrorReport(Box<protocol::Error>),
}

#[derive(Clone, Debug, thiserror::Error, Serialize, Deserialize)]
#[error("placeholder")]
pub struct ActuateAffectorError;

#[derive(Clone, Debug, thiserror::Error, Serialize, Deserialize)]
#[error("placeholder")]
pub struct SubscribeError;

#[derive(Clone, Debug, thiserror::Error, Serialize, Deserialize)]
#[error("placeholder")]
pub struct ServerError;
