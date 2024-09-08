use rpc::client::{RpcClient, RpcError};
use tokio::net::ToSocketAddrs;

use super::Response;
use crate::api;

#[derive(Debug, thiserror::Error)]
pub enum Error<T: std::error::Error> {
    #[error("Server ran into an specific error with our request: {0}")]
    Request(T),
    #[error("Error while communicating with server: {0}")]
    Comms(#[from] RpcError),
}

pub struct Client(rpc::client::RpcClient<super::Request, super::Response>);

impl Client {
    pub async fn connect(
        addr: impl ToSocketAddrs,
        name: String,
    ) -> Result<Self, rpc::client::ConnectError> {
        let rpc_client = RpcClient::connect(addr, name).await?;
        Ok(Self(rpc_client))
    }

    pub async fn list_data(&mut self) -> Result<Vec<protocol::Reading>, RpcError> {
        let request = super::Request::ListData;
        match self.0.send_receive(request.clone()).await? {
            Response::ListData(list) => Ok(list),
            response => Err(RpcError::IncorrectResponse {
                request: format!("{request:?}"),
                response: format!("{response:?}"),
            }),
        }
    }

    pub async fn get_data(
        &mut self,
        start: jiff::Timestamp,
        end: jiff::Timestamp,
        reading: protocol::Reading,
        n: usize,
    ) -> Result<api::Data, Error<api::GetDataError>> {
        let request = super::Request::GetData {
            reading,
            start,
            end,
            n,
        };
        match self.0.send_receive(request.clone()).await? {
            Response::GetData(Ok(data)) => Ok(data),
            Response::GetData(Err(err)) => Err(Error::Request(err)),
            response => Err(Error::Comms(RpcError::IncorrectResponse {
                request: format!("{request:?}"),
                response: format!("{response:?}"),
            })),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetDataError {
    #[error("could not find timestamp in this series")]
    NotFound,
    #[error("data file is empty")]
    EmptyFile,
    #[error("no data to return as the start time is after the last time in the data")]
    StartAfterData,
    #[error("no data to return as the stop time is before the data")]
    StopBeforeData,
    #[error("Error while communicating with server: {0}")]
    Comms(#[from] RpcError),
}
