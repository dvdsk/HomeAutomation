use std::net::SocketAddr;

use data_server::api::Client;
use protocol::Affector;
use tokio::sync::mpsc;
use tracing::error;

use crate::client_name;

pub(crate) async fn watch_and_send(data_server: SocketAddr, mut rx: mpsc::Receiver<Affector>) {
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

        if let Some(mut client) = connected_client.take() {
            if let Err(e) = client.actuate_affector(order).await {
                error!("Could not actuate affector: {e:?}");
            }
            connected_client = Some(client)
        } else {
            if let Ok(c) = Client::connect(data_server, client_name()).await {
                connected_client = Some(c)
            }
        }
    }
}
