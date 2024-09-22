use std::ops::RangeInclusive;
use std::time::Duration;

use protocol::Device;
use rpc::client::RpcClient;
use rpc::client::RpcError;
use tokio::net::ToSocketAddrs;
use tokio::time::sleep;
use tracing::instrument;

use crate::api::Percentile;

use super::ErrorEvent;
use super::GetLogResponse;
use super::GetStatsError;
use super::Response;

pub struct Client(rpc::client::RpcClient<super::Request, super::Response>);
pub use rpc::client::ConnectError;

#[derive(Debug, thiserror::Error)]
pub enum Error<T> {
    #[error("Server ran into an specific error with our request: {0}")]
    Request(T),
    #[error("Error while communicating with server: {0}")]
    Comms(#[from] RpcError),
}

impl Client {
    pub async fn connect(
        addr: impl ToSocketAddrs,
        name: String,
    ) -> Result<Self, rpc::client::ConnectError> {
        let rpc_client = RpcClient::connect(addr, name).await?;
        Ok(Self(rpc_client))
    }

    pub async fn get_percentiles(
        &mut self,
        device: protocol::Device,
    ) -> Result<Vec<Percentile>, Error<GetStatsError>> {
        let request = super::Request::GetStats(device);
        match self.0.send_receive(request.clone()).await? {
            Response::GetStats(Ok(percentiles)) => Ok(percentiles),
            Response::GetStats(Err(e)) => Err(Error::Request(e)),
            response => Err(Error::Comms(RpcError::IncorrectResponse {
                request: format!("{request:?}"),
                response: format!("{response:?}"),
            })),
        }
    }

    #[instrument(err, skip(self))]
    pub async fn get_logs(
        &mut self,
        device: protocol::Device,
        mut range: RangeInclusive<jiff::Timestamp>,
    ) -> Result<Vec<ErrorEvent>, Error<String>> {
        tracing::debug!("get all logs between: {range:?}");
        let mut all = Vec::new();

        while !range.is_empty() {
            let request = super::Request::GetLog {
                device: device.clone(),
                range: range.clone(),
            };
            let partial = match self.0.send_receive(request.clone()).await? {
                Response::GetLog(GetLogResponse::All(log)) => {
                    all.extend_from_slice(&log);
                    return Ok(all);
                }
                Response::GetLog(GetLogResponse::Partial(log)) => log,
                Response::GetLog(GetLogResponse::Err(e)) => return Err(Error::Request(e)),
                response => {
                    return Err(Error::Comms(RpcError::IncorrectResponse {
                        request: format!("{request:?}"),
                        response: format!("{response:?}"),
                    }))
                }
            };

            let partial_ends = partial
                .last()
                .expect("if log.len() == 0 then response is GetLogResponse::All")
                .start;
            range = RangeInclusive::new(partial_ends + jiff::Span::new().seconds(1), *range.end());
            tracing::debug!("Got logs up till {partial_ends}, next requesting: {range:?}");
            all.extend_from_slice(&partial);
            // do not overburden the server
            sleep(Duration::from_millis(100)).await;
        }
        Ok(all)
    }

    pub async fn list_devices(&mut self) -> Result<Vec<Device>, Error<String>> {
        let request = super::Request::ListDevices;
        match self.0.send_receive(request.clone()).await? {
            Response::ListDevices(logs) => Ok(logs),
            response => Err(Error::Comms(RpcError::IncorrectResponse {
                request: format!("{request:?}"),
                response: format!("{response:?}"),
            })),
        }
    }
}
