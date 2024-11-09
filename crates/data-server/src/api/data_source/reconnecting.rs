use protocol::Affector;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};
use tracing::{info, warn};

use super::SendPreEncodedError;

pub struct Client {
    retry_period: Duration,
    conn: Option<(super::Sender, Option<AbortOnDrop>)>,
    addr: SocketAddr,
    affectors: Vec<protocol::Affector>,
    affector_tx: Option<mpsc::Sender<Affector>>,
}

struct AbortOnDrop(JoinHandle<()>);
impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

impl Client {
    /// Needs a list of the affectors that can be controlled through this
    /// node as an argument. If your node provides not controllable affectors
    /// pass in an empty Vec.
    #[must_use]
    pub fn new(
        addr: SocketAddr,
        affectors: Vec<protocol::Affector>,
        affector_tx: Option<mpsc::Sender<Affector>>,
    ) -> Self {
        Self {
            retry_period: Duration::from_millis(200),
            conn: None,
            addr,
            affectors,
            affector_tx,
        }
    }

    async fn send_bytes(&mut self, bytes: &[u8]) {
        loop {
            let (mut sender, handle) = if let Some((sender, handle)) = self.conn.take() {
                dbg!();
                (sender, handle)
            } else {
                dbg!();
                let conn = reconnect(self.addr, &self.affectors, &mut self.retry_period).await;
                let (sender, receiver) = conn.split();
                let handle = self.affector_tx.clone().map(|tx| {
                    let task = forward_received(receiver, tx);
                    let handle = tokio::spawn(task);
                    AbortOnDrop(handle)
                });
                (sender, handle)
            };

            dbg!();
            match sender.send_bytes(bytes).await {
                Ok(_) => {
                    self.retry_period /= 2;
                    self.retry_period = self.retry_period.max(Duration::from_millis(200));
                    self.conn = Some((sender, handle));
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

async fn forward_received(mut receiver: super::Receiver, tx: mpsc::Sender<Affector>) {
    loop {
        let Ok(order) = receiver.receive().await else {
            return;
        };

        let Ok(_) = tx.send(order).await else {
            return;
        };
    }
}

async fn reconnect(
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
