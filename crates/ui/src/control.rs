use std::net::SocketAddr;
use std::time::Duration;

use data_server::api::subscriber::client::Error;
use data_server::api::subscriber::{AffectorError, Client};
use protocol::Affector;
use std::sync::mpsc as std_mpsc;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::error;

use crate::{client_name, Update};

#[derive(Debug)]
pub enum AffectorStatus {
    Send,
    RateLimited,
    NodeOffline,
    /// connection to data_server is/was down, retrying
    ConnIssues,
}

impl std::fmt::Display for AffectorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AffectorStatus::Send => f.write_str("Send succesfully"),
            AffectorStatus::RateLimited => f.write_str(
                "Dropped, affector is rate limited, try again later",
            ),
            AffectorStatus::NodeOffline => {
                f.write_str("Dropped, node is offline")
            }
            AffectorStatus::ConnIssues => {
                f.write_str("Queued, trying to resolve connection issues")
            }
        }
    }
}

pub async fn watch_and_send(
    data_server: SocketAddr,
    mut rx: mpsc::Receiver<Affector>,
    tx: std_mpsc::Sender<Update>,
) {
    let mut connected_client: Option<Client> = None;
    let mut queued = None;

    loop {
        let order = match queued.take() {
            Some(order) => order,
            None => match rx.recv().await {
                Some(order) => order,
                None => return,
            },
        };

        tracing::debug!("got affect order: {order:?}");
        if let Some(mut client) = connected_client.take() {
            let status = match client.actuate_affector(order).await {
                Ok(()) => {
                    connected_client = Some(client);
                    AffectorStatus::Send
                }
                Err(Error::Request(AffectorError::RateLimited)) => {
                    connected_client = Some(client);
                    AffectorStatus::RateLimited
                }
                Err(Error::Request(AffectorError::Offline)) => {
                    connected_client = Some(client);
                    AffectorStatus::NodeOffline
                }
                Err(e) => {
                    queued = Some(order);
                    error!(
                        "Could not actuate affector, error interfacing with data-server, \
                        queing order for when its resolved: {e:?}"
                    );
                    AffectorStatus::ConnIssues
                }
            };
            tx.send(Update::AffectorOrderStatus {
                affector: order.clone(),
                status,
            })
            .expect("Update listener only stops on program exit");
        } else {
            match Client::connect(data_server, client_name()).await {
                Ok(c) => {
                    queued = Some(order);
                    connected_client = Some(c)
                }
                Err(e) => {
                    error!("Could not connect to data_server: {e}");
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
}
