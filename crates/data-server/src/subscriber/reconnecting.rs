use super::{AsyncSubscriber, SubMessage, SubscribeError};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{info, warn};

#[derive(Debug)]
pub struct Subscriber {
    retry_period: Duration,
    connection: Option<AsyncSubscriber>,
    addr: SocketAddr,
    name: String,
}

impl Subscriber {
    pub fn new(addr: SocketAddr, name: String) -> Self {
        Self {
            retry_period: Duration::from_millis(200),
            connection: None,
            addr,
            name,
        }
    }

    /// # Cancel safety
    /// This is cancel safe however the connection will need to be re-established
    /// the next time its called
    pub async fn next_msg(&mut self) -> SubMessage {
        loop {
            let mut conn = if let Some(conn) = self.connection.take() {
                conn
            } else {
                self.get_connection_or_reconnect().await
            };

            match conn.next_msg().await {
                Ok(msg) => {
                    self.retry_period /= 2;
                    self.retry_period = self.retry_period.max(Duration::from_millis(200));
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

    async fn get_connection_or_reconnect(&mut self) -> AsyncSubscriber {
        loop {
            match timeout(
                Duration::from_millis(500),
                AsyncSubscriber::connect(self.addr, &self.name),
            )
            .await
            {
                Ok(Ok(conn)) => {
                    info!("Successfully (re)connected to data-server");
                    return conn;
                }
                Ok(Err(e)) => warn!(
                    "Failed to (re)connect: {e}\n\
                        retrying in {:?}",
                    self.retry_period
                ),
                Err(_) => warn!(
                    "Failed to (re)connect, timed out.\n\
                        retrying in {:?}",
                    self.retry_period
                ),
            }
            sleep(self.retry_period).await;
            self.retry_period = (self.retry_period * 2).min(Duration::from_secs(5));
        }
    }
}
