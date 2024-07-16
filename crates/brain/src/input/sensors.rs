use std::net::SocketAddr;

use data_server::subscriber::SubscribeError;
use data_server::{AsyncSubscriber, SubMessage};
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{error, warn};

use crate::controller::Event;

pub async fn subscribe(event_tx: broadcast::Sender<Event>, data_server: SocketAddr) {
    loop {
        let mut sub = loop {
            match AsyncSubscriber::connect(data_server, "ha-brain").await {
                Ok(sub) => break sub,
                Err(SubscribeError::DecodeFailed(e)) => {
                    error!("Decode failed: {e:?}. is protocol lib up to date? Exiting");
                    return;
                }
                Err(other) => {
                    warn!("Error subscribing to sensor readings: {other}, retrying...");
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        };

        loop {
            match sub.next_msg().await {
                Ok(SubMessage::Reading(reading)) => {
                    event_tx.send(Event::Sensor(reading)).unwrap();
                }
                Ok(SubMessage::ErrorReport(_)) => continue,
                Err(e) => {
                    warn!("Error while subscribed to sensor readings: {e}, retrying...");
                    break;
                }
            };
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
