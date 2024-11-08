use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{info, warn};

use super::SendPreEncodedError;

pub struct Client {
    retry_period: Duration,
    connection: Option<super::Client>,
    addr: SocketAddr,
    affectors: Vec<protocol::Affector>,
}

impl Client {
    /// Needs a list of the affectors that can be controlled through this
    /// node as an argument. If your node provides not controllable affectors
    /// pass in an empty Vec.
    #[must_use]
    pub fn new(addr: SocketAddr, affectors: Vec<protocol::Affector>) -> Self {
        Self {
            retry_period: Duration::from_millis(200),
            connection: None,
            addr,
            affectors,
        }
    }

    async fn send_bytes(&mut self, bytes: &[u8]) {
        loop {
            let mut conn = if let Some(conn) = self.connection.take() {
                conn
            } else {
                get_conn_or_reconnect(self.addr, &self.affectors, &mut self.retry_period).await
            };

            match conn.send_bytes(bytes).await {
                Ok(msg) => {
                    self.retry_period /= 2;
                    self.retry_period = self.retry_period.max(Duration::from_millis(200));
                    self.connection = Some(conn);
                    return msg;
                }
                Err(issue) => {
                    warn!("Conn issue while sending new reading: {issue}, reconnecting");
                }
            };
        }
    }

    /// # Cancel safety
    /// This is cancel safe however the connection will need to be re-established
    /// the next time its called. This will retry forever, you should call this
    /// in a timeout future if that is a problem.
    pub async fn send_reading(&mut self, reading: protocol::Reading) {
        let mut readings = protocol::SensorMessage::<1>::default();
        readings
            .values
            .push(reading)
            .expect("capacity allows one push");
        let msg = protocol::Msg::Readings(readings);
        let bytes = msg.encode();

        self.send_bytes(&bytes).await
    }

    /// # Cancel safety
    /// This is cancel safe however the connection will need to be re-established
    /// the next time its called. This will retry forever, you should call this
    /// in a timeout future if that is a problem.
    pub async fn send_error(&mut self, report: protocol::Error) {
        let msg = protocol::Msg::<1>::ErrorReport(protocol::ErrorReport::new(report));
        let bytes = msg.encode();

        self.send_bytes(&bytes).await
    }

    pub async fn check_send_encoded(&mut self, msg: &[u8]) -> Result<(), SendPreEncodedError> {
        protocol::Msg::<50>::decode(msg.to_vec()).map_err(SendPreEncodedError::EncodingCheck)?;
        self.send_bytes(msg).await;
        Ok(())
    }
}

async fn get_conn_or_reconnect(
    addr: SocketAddr,
    affectors: &[protocol::Affector],
    retry_period: &mut Duration,
) -> super::Client {
    loop {
        match timeout(
            Duration::from_millis(500),
            super::Client::connect(addr, affectors.to_vec()),
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
