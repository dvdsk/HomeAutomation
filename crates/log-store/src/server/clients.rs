use super::db::{Logs, Stats};
use crate::api::{self, ServerError};

pub(crate) async fn handle(port: u16, stats: Stats, logs: Logs) -> color_eyre::Result<()> {
    rpc::server::run(port, move |req| {
        let stats = stats.clone();
        let logs = logs.clone();
        perform_request(req, stats, logs)
    })
    .await
}

async fn perform_request(
    request: api::Request,
    stats: Stats,
    logs: Logs,
) -> Result<api::Response, ServerError> {
    Ok(match request {
        api::Request::Handshake { .. } => return Err(ServerError::AlreadyConnected),
        api::Request::GetLog(device) => api::Response::GetLog(logs.get(&device).await),
        api::Request::GetStats(device) => api::Response::GetStats(stats.get(&device).await),
        api::Request::ListDevices => api::Response::ListDevices(logs.list_devices().await),
    })
}
