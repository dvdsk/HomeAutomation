use color_eyre::Result;
use futures_concurrency::future::Race;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

mod clients;
mod db;

// used from main and tests
pub async fn run(
    data_server: SocketAddr,
    client_port: u16,
    data_dir: &Path,
) -> Result<()> {
    let data = crate::data::Data(Arc::new(Mutex::new(HashMap::new())));

    let error = (
        db::run(data_server, data.clone(), data_dir),
        clients::handle(client_port, data),
    )
        .race()
        .await;
    assert!(
        error.is_err(),
        "db::run and client::handle never return unless an error happens"
    );
    error
}
