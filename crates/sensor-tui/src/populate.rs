use std::net::SocketAddr;
use std::sync::mpsc;

use color_eyre::Result;
use tracing::warn;

use crate::{client_name, Update};

pub fn tree(
    data_store_addr: SocketAddr,
    log_store_addr: SocketAddr,
    tx: mpsc::Sender<Result<Update>>,
) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    let (res1, res2) = rt.block_on(async {
        tokio::join!(
            list_from_store(data_store_addr, tx.clone()),
            list_from_logs(log_store_addr, tx)
        )
    });

    if let Err(e) = res1 {
        warn!("Could not populate readings list from store: {e}")
    }

    if let Err(e) = res2 {
        warn!("Could not populate readings list from logs: {e}")
    }
}

async fn list_from_store(
    data_store_addr: SocketAddr,
    tx: mpsc::Sender<Result<Update>>,
) -> Result<()> {
    let mut client = data_store::api::Client::connect(data_store_addr, client_name()).await?;
    let list = client.list_data().await?;
    tracing::debug!("list: {list:?}");
    tx.send(Ok(Update::ReadingList(list))).unwrap();
    Ok(())
}

async fn list_from_logs(
    log_store_addr: SocketAddr,
    tx: mpsc::Sender<Result<Update>>,
) -> Result<()> {
    let mut client = log_store::api::Client::connect(log_store_addr, client_name()).await?;
    let list = client.list_devices().await?;
    tracing::debug!("list: {list:?}");
    tx.send(Ok(Update::DeviceList(list))).unwrap();
    Ok(())
}
