use std::time::Duration;

use serde::{Deserialize, Serialize};

pub mod client;
pub mod server;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Request<R> {
    Handshake { client_name: String },
    Rpc(R),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Response<V, E>
where
    V: Serialize,
    E: Serialize,
{
    HandshakeOk,
    TooManyReq { allowed_in: Duration },
    RpcResponse(V),
    ServerError(E),
}
