use std::net::SocketAddr;

use data_server::api::subscriber::{ReconnectingClient, SubMessage};
use tokio::sync::broadcast;

use crate::controller::Event;

pub async fn subscribe(
    event_tx: broadcast::Sender<Event>,
    data_server: SocketAddr,
) {
    let mut sub =
        ReconnectingClient::new(data_server, "ha-brain".to_owned()).subscribe();
    loop {
        match sub.next().await {
            SubMessage::Reading(reading) => {
                event_tx.send(Event::Sensor(reading)).unwrap();
            }
            SubMessage::ErrorReport(_)
            | SubMessage::AffectorControlled { .. } => continue,
        }
    }
}
