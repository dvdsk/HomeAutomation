use std::future::Future;
use std::net::SocketAddr;
use std::sync::mpsc;

use color_eyre::Result;
use data_server::api::subscriber;

use crate::{client_name, Update};

pub async fn send_result(
    f: impl Future<Output = Result<Update>>,
    err_text: &'static str,
    tx: mpsc::Sender<Update>,
) {
    match f.await {
        Ok(update) => tx.send(update),
        Err(e) => tx.send(Update::PopulateError(e.wrap_err(err_text))),
    }
    .expect("ui should be listening for these updates")
}

pub async fn tree(
    data_server_addr: SocketAddr,
    data_store_addr: SocketAddr,
    log_store_addr: SocketAddr,
    tx: mpsc::Sender<Update>,
) {
    let _ = tokio::join!(
        send_result(
            list_from_server(data_server_addr),
            "Could not get affector list from data-server",
            tx.clone()
        ),
        send_result(
            list_from_store(data_store_addr),
            "COult not get reading list from data-store",
            tx.clone()
        ),
        send_result(
            list_from_logs(log_store_addr),
            "Could not get logs and histogram from log-store",
            tx
        )
    );
}

async fn list_from_server(data_server_addr: SocketAddr) -> Result<Update> {
    let mut client = subscriber::Client::connect(data_server_addr, client_name()).await?;
    let list = client.list_affectors().await?;
    tracing::debug!("affector list: {list:?}");
    Ok(Update::AffectorList(list))
}

async fn list_from_store(data_store_addr: SocketAddr) -> Result<Update> {
    let mut client = data_store::api::Client::connect(data_store_addr, client_name()).await?;
    let list = client.list_data().await?;
    tracing::debug!("data list: {list:?}");
    Ok(Update::ReadingList(list))
}

async fn list_from_logs(log_store_addr: SocketAddr) -> Result<Update> {
    let mut client = log_store::api::Client::connect(log_store_addr, client_name()).await?;
    let list = client.list_devices().await?;
    tracing::debug!("log list: {list:?}");
    Ok(Update::DeviceList(list))
}
