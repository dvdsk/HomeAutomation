use std::time::Duration;

use serde::{Deserialize, Serialize};

pub mod client;
pub mod server;

// 8 MB
pub(crate) const MAX_PACKAGE_SIZE: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Request<R> {
    Handshake { client_name: String },
    Rpc(R),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Response<V>
where
    V: Serialize,
{
    HandshakeOk,
    AlreadyConnected,
    TooManyReq { allowed_in: Duration },
    RpcResponse(V),
}
