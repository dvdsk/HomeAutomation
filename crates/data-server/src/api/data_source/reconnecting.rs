use futures_concurrency::future::Race;
use protocol::Affector;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};
use tracing::{info, warn};

use super::{ReceiveError, SendPreEncodedError};

pub struct Client {
    conn_handler: AbortOnDrop,
    to_send_tx: mpsc::Sender<(Vec<u8>, SendFeedback)>,
}

struct AbortOnDrop(JoinHandle<()>);
impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

type SendFeedback = oneshot::Sender<Result<(), String>>;
async fn handle_conn(
    addr: SocketAddr,
    affectors: Vec<protocol::Affector>,
    mut msgs_to_send: mpsc::Receiver<(Vec<u8>, SendFeedback)>,
    msgs_recieved: Option<mpsc::Sender<Affector>>,
) {
    loop {
        let mut retry_period = Duration::from_millis(200);
        let conn = reconnect(addr, &affectors, &mut retry_period).await;
        let (sender, receiver) = conn.split();

        if let Some(ref msgs_recieved) = msgs_recieved {
            let res = (
                handle_sending(sender, &mut msgs_to_send),
                handle_recieving(receiver, msgs_recieved),
            )
                .race()
                .await;
            match res {
                SendOrRecvError::Sending(e) => todo!(),
                SendOrRecvError::Recieving(e) => todo!(),
            }
        } else {
            handle_sending(sender, &mut msgs_to_send).await;
        }
    }
}

enum SendOrRecvError {
    Sending(std::io::Error),
    Recieving(ReceiveError),
}

async fn handle_sending(
    mut sender: super::Sender,
    msgs_to_send: &mut mpsc::Receiver<(Vec<u8>, SendFeedback)>,
) -> SendOrRecvError {
    loop {
        let (msg, feedback_channel) = msgs_to_send
            .recv()
            .await
            .expect("this is canceled before the corrosponding sender is dropped");
        let res = sender.send_bytes(&msg).await;
        let _ignore_err = feedback_channel.send(res.as_ref().map_err(|e| e.to_string()).copied());
        if let Err(e) = res {
            return SendOrRecvError::Sending(e);
        }
    }
}

async fn handle_recieving(
    mut receiver: super::Receiver,
    msgs_recieved: &mpsc::Sender<Affector>,
) -> SendOrRecvError {
    loop {
        let msg = match receiver.receive().await {
            Ok(msg) => msg,
            Err(err) => return SendOrRecvError::Recieving(err),
        };

        msgs_recieved
            .send(msg)
            .await
            .expect("Reciever in the 'Client' is never dropped")
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
        let (to_send_tx, to_send_rx) = mpsc::channel(100);
        let task = handle_conn(addr, affectors, to_send_rx, affector_tx);
        let handle = tokio::spawn(task);
        Self {
            conn_handler: AbortOnDrop(handle),
            to_send_tx,
        }
    }

    async fn send_bytes(&mut self, bytes: Vec<u8>) -> Result<(), ()> {
        let (tx, rx) = oneshot::channel();
        self.to_send_tx.send((bytes, tx)).await;
        rx.await
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
