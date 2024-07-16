use super::{AsyncSubscriber, SubMessage, SubscribeError};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{info, warn};

pub struct Subscriber {
    connection: Option<AsyncSubscriber>,
    addr: SocketAddr,
    name: String,
}

impl Subscriber {
    pub fn new(addr: SocketAddr, name: String) -> Self {
        Self {
            connection: None,
            addr,
            name,
        }
    }

    pub async fn next_msg(&mut self) -> SubMessage {
        loop {
            let mut retry_period = Duration::from_millis(50);
            let mut conn = loop {
                if let Some(conn) = self.connection.take() {
                    break conn;
                }

                match timeout(
                    Duration::from_millis(500),
                    AsyncSubscriber::connect(self.addr, &self.name),
                )
                .await
                {
                    Ok(Ok(conn)) => {
                        info!("Successfully (re)connected to data-server");
                        break conn;
                    }
                    Ok(Err(e)) => warn!(
                        "Failed to (re)connect: {e}\n\
                        retrying in {retry_period:?}"
                    ),
                    Err(_) => warn!(
                        "Failed to (re)connect, timed out.\n\
                        retrying in {retry_period:?}"
                    ),
                }
                sleep(retry_period).await;
                retry_period = (retry_period * 2).min(Duration::from_secs(5));
            };

            match conn.next_msg().await {
                Ok(msg) => {
                    self.connection = Some(conn);
                    return msg;
                }
                Err(SubscribeError::DecodeFailed(e)) => {
                    panic!("Critical error while receiving msg from data-server: {e}")
                }
                Err(issue) => {
                    warn!("Conn issue while getting next_msg: {issue}, reconnecting")
                }
            };
        }
    }
}
