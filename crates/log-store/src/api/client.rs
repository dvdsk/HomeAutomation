use protocol::Device;
use rpc::client::RpcClient;
use rpc::client::RpcError;
use tokio::net::ToSocketAddrs;

use crate::api::Percentile;

use super::ErrorEvent;
use super::GetLogError;
use super::GetStatsError;
use super::Response;

pub struct Client(rpc::client::RpcClient<super::Request, super::Response>);

#[derive(Debug, thiserror::Error)]
pub enum Error<T: std::error::Error> {
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

    pub async fn get_logs(
        &mut self,
        device: protocol::Device,
    ) -> Result<Vec<ErrorEvent>, Error<GetLogError>> {
        let request = super::Request::GetLog(device);
        match self.0.send_receive(request.clone()).await? {
            Response::GetLog(Ok(log)) => Ok(log),
            Response::GetLog(Err(e)) => Err(Error::Request(e)),
            response => Err(Error::Comms(RpcError::IncorrectResponse {
                request: format!("{request:?}"),
                response: format!("{response:?}"),
            })),
        }
    }

    pub async fn list_devices(&mut self) -> Result<Vec<Device>, Error<GetLogError>> {
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
