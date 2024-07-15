use data_server::subscriber::SubscribeError;
use tokio::time::sleep;
use tracing::warn;
use std::time::Duration;
use data_server::{AsyncSubscriber, SubMessage};
use std::net::SocketAddr;

pub(crate) struct ReconnectingSubscriber {
    pub(crate) connection: Option<AsyncSubscriber>,
    pub(crate) addr: SocketAddr,
}

impl ReconnectingSubscriber {
    pub(crate) fn new(addr: SocketAddr) -> Self {
        Self {
            connection: None,
            addr,
        }
    }

    pub(crate) async fn next_msg(&mut self) -> SubMessage {
        loop {
            let mut retry_period = Duration::from_millis(50);
            let mut conn = loop {
                if let Some(conn) = self.connection.take() {
                    break conn;
                }

                match AsyncSubscriber::connect(self.addr).await {
                    Ok(conn) => break conn,
                    Err(e) => warn!("Failed to (re)connect: {e}"),
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
                    warn!("reconnecting, conn issue while getting next_msg: {issue}")
                }
            };
        }
    }
}

