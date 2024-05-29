use std::net::SocketAddr;

use data_server::{AsyncSubscriber, SubscribeError};
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{error, warn};

use crate::controller::Event;

pub async fn subscribe(event_tx: broadcast::Sender<Event>, subscribe_port: u16) {
    loop {
        let mut sub = loop {
            let addr = SocketAddr::from(([127, 0, 0, 1], subscribe_port));
            match AsyncSubscriber::connect(addr).await {
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
            match sub.next().await {
                Ok(Ok(reading)) => event_tx.send(Event::Sensor(reading)).unwrap(),
                Ok(Err(_)) => continue,
                Err(e) => {
                    warn!("Error while subscribed to sensor readings: {e}, retrying...");
                    break;
                }
            };
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
