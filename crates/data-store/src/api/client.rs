use rpc::client::{RpcClient, RpcError};
use tokio::net::ToSocketAddrs;

use super::Response;

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
    ) -> Result<(Vec<jiff::Timestamp>, Vec<f32>), RpcError> {
        let request = super::Request::GetData {
            reading,
            start,
            end,
            n,
        };
        match self.0.send_receive(request.clone()).await? {
            Response::GetData { time, data } => Ok((time, data)),
            response => Err(RpcError::IncorrectResponse {
                request: format!("{request:?}"),
                response: format!("{response:?}"),
            }),
        }
    }
}
