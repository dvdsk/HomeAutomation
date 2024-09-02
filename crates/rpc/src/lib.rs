use std::future;
use std::marker::PhantomData;
use std::time::Duration;

use futures::{stream, Stream};
use serde::{Deserialize, Serialize};

pub mod client;
pub mod server;

// 8 MB
pub(crate) const MAX_PACKAGE_SIZE: usize = 8 * 1024 * 1024;

pub trait SubscriberHandler: Send + 'static {
    type Update;
    #[allow(async_fn_in_trait)]
    fn setup(
        &mut self,
    ) -> impl std::future::Future<Output = impl Stream<Item = Self::Update> + Send + 'static>
           + Send
           + 'static;
}

/// needed (I know ugly) to give the Option::None a complete type
/// for the server::run function (which takes a Option<impl SubscriberHandler>)
#[derive(Debug, Clone)]
pub struct SubscribersUnsupported<Update> {
    phantom: PhantomData<Update>,
}

impl<Update: std::marker::Send + 'static> SubscriberHandler for SubscribersUnsupported<Update> {
    type Update = Update;

    fn setup(
        &mut self,
    ) -> impl std::future::Future<
        Output = impl futures::prelude::Stream<Item = Self::Update> + Send + 'static,
    > + Send
           + 'static {
        future::pending::<stream::Empty<Update>>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request<R> {
    Handshake { client_name: String },
    Subscribe,
    Rpc(R),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response<V>
where
    V: Serialize,
{
    HandshakeOk,
    SubscribeOk,
    AlreadyConnected,
    TooManyReq { allowed_in: Duration },
    RpcResponse(V),
    Update(V),
}
