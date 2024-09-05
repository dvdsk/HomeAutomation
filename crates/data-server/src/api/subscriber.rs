use serde::{Deserialize, Serialize};
pub use client::reconnecting::Client as ReconnectingClient;
pub use client::reconnecting::SubscribedClient as ReconnectingSubscribedClient;
pub use client::Client;
pub use client::Subscribed as SubscribedClient;
pub mod client;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Handshake { name: String },
    Actuate(protocol::Affector),
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
    // An affector was moved/updated
    AffectorControlled {
        affector: protocol::Affector,
        controlled_by: String,
    },
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
pub enum ServerError {
    #[error("Could not activate affector")]
    FailedToSpread,
}

