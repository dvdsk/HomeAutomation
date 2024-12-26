use futures_concurrency::future::Race;
use protocol::Affector;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio::time::{sleep, timeout};
use tracing::{info, warn};

use super::SendPreEncodedError;

pub struct Client {
    _conn_handler_task: AbortOnDrop,
    to_send_tx: mpsc::Sender<SendItem>,
}

struct AbortOnDrop(JoinHandle<()>);
impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn handle_conn(
    addr: SocketAddr,
    affectors: Vec<protocol::Affector>,
    mut msgs_to_send: mpsc::Receiver<SendItem>,
    msgs_recieved: Option<mpsc::Sender<Affector>>,
) {
    loop {
        let mut retry_period = Duration::from_millis(200);
        let conn = reconnect(addr, &affectors, &mut retry_period).await;

        if let Some(ref msgs_recieved) = msgs_recieved {
            (
                handle_sending(conn.sender, &mut msgs_to_send),
                handle_recieving(conn.receiver, msgs_recieved),
            )
                .race()
                .await;
        } else {
            handle_sending(conn.sender, &mut msgs_to_send).await;
        }
    }
}

struct SendItem {
    bytes: Vec<u8>,
    feedback: oneshot::Sender<Result<(), SendError>>,
    deadline: Instant,
}

async fn handle_sending(
    mut sender: super::Sender,
    msgs_to_send: &mut mpsc::Receiver<SendItem>,
) {
    loop {
        let item = msgs_to_send.recv().await.expect(
            "this is canceled before the corrosponding sender is dropped",
        );
        let res = if item.deadline.elapsed().is_zero() {
            sender
                .send_bytes(&item.bytes)
                .await
                .map_err(|e| e.to_string())
                .map_err(SendError::Io)
        } else {
            Err(SendError::Outdated)
        };
        let _ignore_err = item.feedback.send(res.clone());
        if let Err(e) = res {
            tracing::error!("Error sending reading or sensor error: {e}");
            return;
        }
        tracing::debug!("send bytes: {}", item.bytes.len());
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum SendError {
    #[error("Io error while sending data to data-server: {0}")]
    Io(String),
    #[error(
        "Sending the data was aborted, probably due to \
        reconnect due to failed recieve"
    )]
    Aborted,
    /// returns the encoded message that you tried to send
    #[error(
        "Sender queue is full, it has probably been disconnected for a while"
    )]
    Overloaded { bytes: Vec<u8> },
    #[error("Could not send in time (deadline expired)")]
    Outdated,
}

async fn handle_recieving(
    mut receiver: super::Receiver,
    msgs_recieved: &mpsc::Sender<Affector>,
) {
    loop {
        let msg = match receiver.receive().await {
            Ok(msg) => msg,
            Err(err) => {
                tracing::error!("Error receiving affector orders: {err}");
                return;
            }
        };

        tracing::debug!("recv item: {msg:?}");
        msgs_recieved
            .send(msg)
            .await
            .expect("Reciever in the 'Client' is never dropped")
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Invalid address: {0}")]
pub struct InvalidAddress(String);

impl Client {
    /// Needs a list of the affectors that can be controlled through this
    /// node as an argument. If your node provides not controllable affectors
    /// pass in an empty Vec.
    ///
    /// # Errors
    /// returns an error if the address could not looked up
    pub async fn new<A: tokio::net::ToSocketAddrs>(
        addr: A,
        affectors: Vec<protocol::Affector>,
        affector_tx: Option<mpsc::Sender<Affector>>,
    ) -> Result<Self, InvalidAddress> {
        let addr: SocketAddr = tokio::net::lookup_host(addr)
            .await
            .map_err(|e| e.to_string())
            .map_err(InvalidAddress)?
            .next()
            .ok_or_else(|| {
                InvalidAddress("No address passed in".to_string())
            })?;

        let (to_send_tx, to_send_rx) = mpsc::channel(100);
        let task = handle_conn(addr, affectors, to_send_rx, affector_tx);
        let handle = tokio::spawn(task);
        Ok(Self {
            _conn_handler_task: AbortOnDrop(handle),
            to_send_tx,
        })
    }

    async fn send_bytes(
        &mut self,
        bytes: Vec<u8>,
        deadline: Instant,
    ) -> Result<(), SendError> {
        let (tx, rx) = oneshot::channel();
        match self.to_send_tx.try_send(SendItem {
            bytes,
            feedback: tx,
            deadline,
        }) {
            Ok(()) => (),
            Err(mpsc::error::TrySendError::Full(SendItem {
                bytes, ..
            })) => return Err(SendError::Overloaded { bytes }),
            Err(mpsc::error::TrySendError::Closed(_)) => {
                panic!("re-connect loop should never end")
            }
        }
        match rx.await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(send_error)) => Err(send_error),
            Err(_) => Err(SendError::Aborted),
        }
    }

    /// # Cancel safety
    /// This is cancel safe however the connection will need to be re-established
    /// the next time its called. This will retry forever, you should call this
    /// in a timeout future if that is a problem.
    pub async fn send_reading(
        &mut self,
        reading: protocol::Reading,
    ) -> Result<(), SendError> {
        let deadline =
            Instant::now() + reading.device().info().temporal_resolution;

        let mut readings = protocol::SensorMessage::<1>::default();
        readings
            .values
            .push(reading)
            .expect("capacity allows one push");
        let msg = protocol::Msg::Readings(readings);
        let bytes = msg.encode();

        self.send_bytes(bytes, deadline).await
    }

    const ERROR_REPORT_SEND_DEADLINE: Duration = Duration::from_secs(60 * 15);
    /// # Cancel safety
    /// This is cancel safe however the connection will need to be re-established
    /// the next time its called. This will retry forever, you should call this
    /// in a timeout future if that is a problem.
    ///
    /// # Warning
    /// Message is not send if it could not be send within 15 minutes. Client
    /// should re-send the error then.
    pub async fn send_error(
        &mut self,
        report: protocol::Error,
    ) -> Result<(), SendError> {
        let msg =
            protocol::Msg::<1>::ErrorReport(protocol::ErrorReport::new(report));
        let bytes = msg.encode();

        self.send_bytes(
            bytes,
            Instant::now() + Self::ERROR_REPORT_SEND_DEADLINE,
        )
        .await
    }

    pub async fn check_send_encoded(
        &mut self,
        msg: Vec<u8>,
    ) -> Result<(), SendPreEncodedError> {
        let decoded = protocol::Msg::<50>::decode(msg.to_vec())
            .map_err(SendPreEncodedError::EncodingCheck)?;
        let deadline = match decoded {
            protocol::Msg::Readings(sensor_message) => sensor_message.values.iter().map(|v| v.device().info().temporal_resolution).min().expect("empty sensormessages are forbidden"),
            protocol::Msg::ErrorReport(_) => Self::ERROR_REPORT_SEND_DEADLINE,
            protocol::Msg::AffectorList(_) => unreachable!("send by client on reconnect only, never send by user of client"),
        };

        self.send_bytes(msg, Instant::now() + deadline)
            .await
            .map_err(SendPreEncodedError::Sending)
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
