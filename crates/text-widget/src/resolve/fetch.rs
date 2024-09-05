use color_eyre::eyre::Context;
use data_server::api::subscriber::ReconnectingSubscribedClient;
use data_server::api::subscriber::SubMessage;
use std::net::SocketAddr;

use data_store::api::Client;

pub(super) async fn datalist_on_store(
    store: SocketAddr,
    name: String,
) -> color_eyre::Result<Vec<protocol::Reading>> {
    Client::connect(store, name)
        .await
        .wrap_err("failed to connect to data store to list data")?
        .list_data()
        .await
        .wrap_err("connected bu list data call failed")
}

pub(super) async fn datalist_from_updates(
    client: &mut ReconnectingSubscribedClient,
    list: &mut Vec<protocol::Reading>,
) {
    loop {
        if let SubMessage::Reading(new) = client.next().await {
            if !list.iter().any(|in_list| new.is_same_as(in_list)) {
                list.push(new);
            }
        }
    }
}
