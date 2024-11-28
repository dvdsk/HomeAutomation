use nusb::transfer;
use tokio::sync::mpsc;

use tokio::time::{sleep, sleep_until, timeout};

use protocol::Affector;
use std::time::Duration;
use std::vec;

use color_eyre::eyre::{bail, Context};

use tokio::time::Instant;

mod device;

pub(crate) struct ReconnectingUsb {
    pub(crate) serial_number: String,
    /// when we are allowed to poll the usb
    /// device for more data again
    pub(crate) next_poll: Instant,
    pub(crate) conn: Option<nusb::Device>,
    pub(crate) bytes: vec::IntoIter<u8>,
    pub(crate) to_send: mpsc::Receiver<Affector>,
}

pub(crate) enum Status {
    Done,
    CallAgain,
}

impl ReconnectingUsb {
    pub(crate) fn new(serial_number: String, to_send: mpsc::Receiver<Affector>) -> Self {
        ReconnectingUsb {
            serial_number,
            next_poll: Instant::now(),
            conn: None,
            bytes: Vec::new().into_iter(),
            to_send,
        }
    }

    pub(crate) async fn get_affectors(&mut self) -> color_eyre::Result<Vec<Affector>> {
        let msg = loop {
            match self
                .try_request_data(device::get(protocol::usb::GET_AFFECTOR_LIST))
                .await
            {
                Ok(Some(msg)) => break msg,
                Ok(None) => continue, // needs another call to decode more
                Err(e) => {
                    tracing::warn!("Error trying get affector list: {e}");
                    sleep(Duration::from_secs(5)).await;
                }
            }
        };
        let msg = protocol::Msg::<50>::decode(msg.to_vec()).wrap_err(
            "Could not decode protocol::Msg, maybe the protocol library \
            has changed since this was compiled",
        )?;
        let protocol::Msg::AffectorList(list) = msg else {
            unreachable!("affector list request is only anwserd by affector list msg");
        };

        Ok(list.values.to_vec())
    }

    pub(crate) async fn handle_usb(&mut self) -> Vec<u8> {
        let mut retry_period = Duration::from_millis(100);
        loop {
            if let Some(len) = self.bytes.next() {
                if len == 0 {
                    self.bytes = Vec::new().into_iter();
                    continue;
                }
                return self.bytes.by_ref().take(len as usize).collect();
            }

            if let Ok(order) = self.to_send.try_recv() {
                self.send_order(&mut retry_period, order).await;
            }

            self.receive_bytes(&mut retry_period).await;
        }
    }

    pub(crate) async fn receive_bytes(&mut self, retry_period: &mut Duration) {
        self.bytes = loop {
            sleep_until(self.next_poll).await;
            self.next_poll = Instant::now() + Duration::from_millis(100);

            match self
                .try_request_data(device::get(protocol::usb::GET_QUEUED_MESSAGES))
                .await
            {
                Ok(Some(bytes)) => break bytes.into_iter(),
                Ok(None) => continue,
                Err(e) => {
                    tracing::warn!("could not receive sensor message: {e:?}");
                    *retry_period *= 2;
                    *retry_period = (*retry_period).min(Duration::from_secs(30));
                    sleep(*retry_period).await;
                }
            };
        };
    }

    pub(crate) async fn send_order(&mut self, retry_period: &mut Duration, order: Affector) {
        tracing::info!("sending order");
        let data = order.encode();
        for _ in 0..2 {
            match self
                .try_send_data(device::send(protocol::usb::AFFECTOR_ORDER, &data))
                .await
            {
                Ok(Status::Done) => break,
                Ok(Status::CallAgain) => continue,
                Err(e) => {
                    tracing::warn!("could not send affector order: {e:?}");
                    *retry_period *= 2;
                    *retry_period = (*retry_period).min(Duration::from_secs(30));
                    sleep(*retry_period).await;
                }
            }
        }
    }

    pub(crate) async fn try_request_data(
        &mut self,
        request: transfer::ControlIn,
    ) -> color_eyre::Result<Option<Vec<u8>>> {
        if let Some(device) = self.conn.take() {
            let do_request = device.control_in(request);
            let msg = match timeout(Duration::from_secs(1), do_request)
                .await
                .map(|res| res.into_result())
            {
                Ok(Ok(msg)) => msg,
                Ok(e @ Err(_)) => e.wrap_err("Something went wrong with control_in request")?,
                Err(_timeout) => bail!("Usb control_in request timed out"),
            };

            self.conn = Some(device);
            return Ok(Some(msg));
        }

        let list = device::list_usb_devices(&self.serial_number)?;
        self.conn = Some(device::get_usb_device(list, &self.serial_number)?);

        Ok(None) // no errors but not done yet, call us again
    }

    pub(crate) async fn try_send_data(
        &mut self,
        request: transfer::ControlOut<'_>,
    ) -> color_eyre::Result<Status> {
        if let Some(device) = self.conn.take() {
            device
                .control_out(request)
                .await
                .into_result()
                .wrap_err("Something went wrong with control_out request")?;

            self.conn = Some(device);
            return Ok(Status::Done);
        }

        let list = device::list_usb_devices(&self.serial_number)?;
        self.conn = Some(device::get_usb_device(list, &self.serial_number)?);

        Ok(Status::CallAgain) // no errors but not done yet, call us again
    }
}
