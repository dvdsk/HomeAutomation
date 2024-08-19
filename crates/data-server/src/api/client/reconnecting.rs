use crate::api::SubMessage;

use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{info, warn};

#[derive(Debug)]
pub struct Client {
    retry_period: Duration,
    connection: Option<super::Client>,
    addr: SocketAddr,
    name: String,
}

impl Client {
    #[must_use]
    pub fn new(addr: SocketAddr, name: String) -> Self {
        Self {
            retry_period: Duration::from_millis(200),
            connection: None,
            addr,
            name,
        }
    }

    #[must_use]
    pub fn subscribe(self) -> SubscribedClient {
        SubscribedClient {
            retry_period: self.retry_period,
            connection: self.connection.map(ConnState::Connected),
            addr: self.addr,
            name: self.name,
        }
    }

    /// # Cancel safety
    /// This is cancel safe however the connection will need to be re-established
    /// the next time its called. This will retry forever, you should call this
    /// in a timeout future.
    pub async fn actuate_affector(&mut self, affector: protocol::Affector) {
        loop {
            let mut conn = if let Some(conn) = self.connection.take() {
                conn
            } else {
                get_connection_or_reconnect(self.addr, &self.name, &mut self.retry_period).await
            };

            match conn.actuate_affector(affector).await {
                Ok(msg) => {
                    self.retry_period /= 2;
                    self.retry_period = self.retry_period.max(Duration::from_millis(200));
                    self.connection = Some(conn);
                    return msg;
                }
                Err(issue) => {
                    warn!("Conn issue while getting next_msg: {issue}, reconnecting");
                }
            };
        }
    }

    /// # Cancel safety
    /// This is cancel safe however the connection will need to be re-established
    /// the next time its called. This will retry forever, you should call this
    /// in a timeout future.
    pub async fn list_affectors(&mut self) -> Vec<protocol::Affector> {
        loop {
            let mut conn = if let Some(conn) = self.connection.take() {
                conn
            } else {
                get_connection_or_reconnect(self.addr, &self.name, &mut self.retry_period).await
            };

            match conn.list_affectors().await {
                Ok(list) => {
                    self.retry_period /= 2;
                    self.retry_period = self.retry_period.max(Duration::from_millis(200));
                    self.connection = Some(conn);
                    return list;
                }
                Err(issue) => {
                    warn!("Conn issue while getting next_msg: {issue}, reconnecting");
                }
            };
        }
    }
}

#[derive(Debug)]
enum ConnState {
    Connected(super::Client),
    Subbed(super::Subscribed),
}

#[derive(Debug)]
pub struct SubscribedClient {
    retry_period: Duration,
    connection: Option<ConnState>,
    addr: SocketAddr,
    name: String,
}

impl SubscribedClient {
    pub async fn next(&mut self) -> SubMessage {
        loop {
            let conn = if let Some(conn) = self.connection.take() {
                conn
            } else {
                ConnState::Connected(
                    get_connection_or_reconnect(self.addr, &self.name, &mut self.retry_period)
                        .await,
                )
            };

            let mut subbed = match conn {
                ConnState::Connected(conn) => match conn.subscribe().await {
                    Ok(subbed) => subbed,
                    Err(_) => continue,
                },
                ConnState::Subbed(subbed) => subbed,
            };

            match subbed.next().await {
                Ok(msg) => {
                    self.retry_period /= 2;
                    self.retry_period = self.retry_period.max(Duration::from_millis(200));
                    self.connection = Some(ConnState::Subbed(subbed));
                    return msg;
                }
                Err(issue) => {
                    warn!("Conn issue while getting next_msg: {issue}, reconnecting");
                }
            };
        }
    }
}

async fn get_connection_or_reconnect(
    addr: SocketAddr,
    name: &str,
    retry_period: &mut Duration,
) -> super::Client {
    loop {
        match timeout(
            Duration::from_millis(500),
            super::Client::connect(addr, name.to_owned()),
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
                retry_period
            ),
            Err(_) => warn!(
                "Failed to (re)connect, timed out.\n\
                    retrying in {:?}",
                retry_period
            ),
        }
        sleep(*retry_period).await;
        *retry_period = (*retry_period * 2).min(Duration::from_secs(5));
    }
}
