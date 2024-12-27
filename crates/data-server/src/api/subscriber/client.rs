use rpc::client::RpcClient;
use rpc::client::RpcError;
use tokio::net::ToSocketAddrs;

use super::AffectorError;
use super::Request;
use super::Response;

use crate::api::subscriber;
pub(crate) mod reconnecting;

#[derive(Debug)]
pub struct Client(rpc::client::RpcClient<super::Request, super::Response>);

impl Client {
    pub async fn connect(
        addr: impl ToSocketAddrs,
        name: String,
    ) -> Result<Self, rpc::client::ConnectError> {
        let rpc_client = RpcClient::connect(addr, name).await?;
        Ok(Self(rpc_client))
    }

    pub async fn actuate_affector(
        &mut self,
        affector: protocol::Affector,
    ) -> Result<(), Error<AffectorError>> {
        let request = Request::Actuate(affector);
        match self.0.send_receive(request.clone()).await? {
            Response::Actuate(res) => res.map_err(Error::Request),
            response => Err(Error::Comms(RpcError::IncorrectResponse {
                request: format!("{request:?}"),
                response: format!("{response:?}"),
            })),
        }
    }

    pub async fn list_affectors(
        &mut self,
    ) -> Result<Vec<protocol::Affector>, Error<AffectorError>> {
        let request = Request::ListAffectors;
        match self.0.send_receive(request.clone()).await? {
            Response::ListAffectors(list) => Ok(list),
            response => Err(Error::Comms(RpcError::IncorrectResponse {
                request: format!("{request:?}"),
                response: format!("{response:?}"),
            })),
        }
    }

    pub async fn subscribe(
        mut self,
    ) -> Result<Subscribed, Error<subscriber::SubscribeError>> {
        self.0.subscribe().await?;
        Ok(Subscribed(self))
    }
}

#[derive(Debug)]
pub struct Subscribed(Client);

impl Subscribed {
    pub async fn next(
        &mut self,
    ) -> Result<subscriber::SubMessage, Error<subscriber::SubscribeError>> {
        let received = self.0 .0.next().await?;

        if let subscriber::Response::SubUpdate(update) = received {
            Ok(update)
        } else {
            Err(Error::Comms(RpcError::IncorrectResponse {
                request: "none, we are subscribed".to_string(),
                response: format!("{received:?}"),
            }))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error<T: std::error::Error> {
    #[error("Error while sending request")]
    Sending(#[source] std::io::Error),
    #[error("Error while sending request")]
    Receiving(#[source] std::io::Error),
    #[error("General error while processing request")]
    Server(#[source] subscriber::ServerError),
    #[error("Server ran into an specific error with our request")]
    Request(#[source] T),
    #[error("Error while communicating with server")]
    Comms(
        #[from]
        #[source]
        RpcError,
    ),
}
