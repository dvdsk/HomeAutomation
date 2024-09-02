use rpc::client::RpcClient;
use tokio::net::ToSocketAddrs;

use super::Response;

pub struct Client(rpc::client::RpcClient<super::Request, super::Response>);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Got unexpected response response to request {request:?}")]
    IncorrectResponse { request: String, response: String },
    #[error("Error while communicating with server: {0}")]
    Comms(#[from] rpc::client::RpcError),
}

impl Client {
    pub async fn connect(
        addr: impl ToSocketAddrs,
        name: String,
    ) -> Result<Self, rpc::client::ConnectError> {
        let rpc_client = RpcClient::connect(addr, name).await?;
        Ok(Self(rpc_client))
    }

    pub async fn list_data(&mut self) -> Result<Vec<protocol::Reading>, Error> {
        let request = super::Request::ListData;
        match self.0.send_receive(request.clone()).await? {
            Response::ListData(list) => Ok(list),
            response => Err(Error::IncorrectResponse {
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
    ) -> Result<(Vec<jiff::Timestamp>, Vec<f32>), Error> {
        let request = super::Request::GetData {
            reading,
            start,
            end,
            n,
        };
        match self.0.send_receive(request.clone()).await? {
            Response::GetData { time, data } => Ok((time, data)),
            response => Err(Error::IncorrectResponse {
                request: format!("{request:?}"),
                response: format!("{response:?}"),
            }),
        }
    }
}
