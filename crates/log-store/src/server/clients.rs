use super::db::{Logs, Stats};
use crate::api::{self, ServerError};

pub(crate) async fn handle(port: u16, stats: Stats, logs: Logs) -> color_eyre::Result<()> {
    rpc::server::run(
        port,
        move |req, _| {
            let stats = stats.clone();
            let logs = logs.clone();
            perform_request(req, stats, logs)
        },
        Option::<rpc::SubscribersUnsupported<api::Response>>::None,
    )
    .await
}

async fn perform_request(request: api::Request, stats: Stats, logs: Logs) -> api::Response {
    match perform_request_inner(request, stats, logs).await {
        Ok(resp) => resp,
        Err(e) => api::Response::Error(e),
    }
}
async fn perform_request_inner(
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
