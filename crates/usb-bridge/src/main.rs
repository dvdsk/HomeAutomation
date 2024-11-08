use std::vec;
use std::{net::SocketAddr, time::Duration};

use clap::Parser;

use color_eyre::eyre::{eyre, Context};
use color_eyre::Section;
use data_server::api::data_source::reconnecting;
use nusb::transfer;
use protocol::{usb, Affector};
use tokio::time::{sleep, sleep_until, Instant};

use ratelimited_logger as rl;

#[derive(Parser)]
#[command(name = "usb-bridge")]
#[command(version = "1.0")]
#[command(about = "forwards sensor info from and affector orders to nodes attached to usb")]
struct Cli {
    /// Where to send the data on the local system
    #[arg(short, long("data-server"), default_value = "192.168.1.43:1234")]
    data_server: SocketAddr,
    /// Serial number of the device to connect case insensitive
    #[arg(short, long)]
    serial_number: String,
}

struct ReconnectingUsbSub {
    serial_number: String,
    /// when we are allowed to poll the usb
    /// device for more data again
    next_poll: Instant,
    conn: Option<nusb::Device>,
    bytes: vec::IntoIter<u8>,
    logger: ratelimited_logger::RateLimitedLogger,
}

fn usb_request(request: u8) -> transfer::ControlIn {
    transfer::ControlIn {
        control_type: transfer::ControlType::Vendor,
        recipient: transfer::Recipient::Interface,
        request,
        value: 0,
        index: 0,
        length: usb::SEND_BUFFER_SIZE.try_into().expect("fits"),
    }
}

impl ReconnectingUsbSub {
    fn new(serial_number: String) -> Self {
        ReconnectingUsbSub {
            serial_number,
            next_poll: Instant::now(),
            conn: None,
            bytes: Vec::new().into_iter(),
            logger: ratelimited_logger::RateLimitedLogger::default(),
        }
    }

    async fn get_affectors(&mut self) -> color_eyre::Result<Vec<Affector>> {
        let msg = loop {
            if let Some(msg) = self
                .try_request_data(usb_request(usb::GET_AFFECTOR_LIST))
                .await?
            {
                break msg;
            }
            sleep(Duration::from_secs(5)).await;
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

    async fn recv_msgs(&mut self) -> Vec<u8> {
        let mut retry_period = Duration::from_millis(100);
        loop {
            if let Some(len) = self.bytes.next() {
                if len == 0 {
                    self.bytes = Vec::new().into_iter();
                    continue;
                }
                return self.bytes.by_ref().take(len as usize).collect();
            }

            self.bytes = loop {
                match self
                    .try_request_data(usb_request(usb::GET_QUEUED_MESSAGES))
                    .await
                {
                    Ok(Some(bytes)) => break bytes.into_iter(),
                    Ok(None) => continue,
                    Err(e) => {
                        let logger = &mut self.logger;
                        rl::warn!(logger; "{e:?}");
                        retry_period *= 2;
                        let retry_period = retry_period.min(Duration::from_secs(30));
                        sleep(retry_period).await;
                    }
                };
            };
        }
    }

    async fn try_request_data(
        &mut self,
        request: transfer::ControlIn,
    ) -> color_eyre::Result<Option<Vec<u8>>> {
        if let Some(device) = self.conn.take() {
            sleep_until(self.next_poll).await;
            self.next_poll = Instant::now() + Duration::from_millis(100);
            let msg = device
                .control_in(request)
                .await
                .into_result()
                .wrap_err("Something went wrong with control_in request")?;

            self.conn = Some(device);
            return Ok(Some(msg));
        }

        let list = list_usb_devices(&self.serial_number)?;
        self.conn = Some(get_usb_device(list, &self.serial_number)?);

        Ok(None) // no errors but not done yet, call us again
    }
}

fn list_usb_devices(serial_number: &str) -> Result<Vec<nusb::DeviceInfo>, color_eyre::eyre::Error> {
    let list: Vec<_> = nusb::list_devices()
        .wrap_err("Could not list usb devices")?
        .filter(|d| {
            d.serial_number()
                .is_some_and(|d| d.eq_ignore_ascii_case(serial_number))
        })
        .collect();
    Ok(list)
}

fn get_usb_device(
    list: Vec<nusb::DeviceInfo>,
    serial_number: &str,
) -> Result<nusb::Device, color_eyre::eyre::Error> {
    match list.as_slice() {
        [dev] => dev,
        [] => {
            return Err(eyre!("No usb device found with the correct serial"))
                .with_note(|| format!("looking for device with serial: {serial_number}"))
                .suggestion(
                    "Is the device working (sometimes programming fails) \
                    & connected?",
                );
        }
        more => {
            return Err(eyre!("Multiple usb devices have the same serial number")
                .with_note(|| format!("they are: {more:?}")));
        }
    }
    .open()
    .wrap_err("Could not open the usb device")
    .suggestion("Try running as sudo")
    .with_suggestion(|| {
        format!(
            "Add a .rules file in /etc/udev/rules.d with line: \
                ATTRS{{serial}}==\"{}\", MODE=\"660\", GROUP=\"{}\", TAG+=\"uaccess\"",
            serial_number,
            users::get_current_groupname()
                .expect("process should always run as a group")
                .to_string_lossy()
        )
    })
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install().unwrap();
    setup_tracing();
    let args = Cli::parse();

    let mut usb = ReconnectingUsbSub::new(args.serial_number);
    let affectors = usb
        .get_affectors()
        .await
        .wrap_err("Could not get affector list")?;
    let mut server_client = reconnecting::Client::new(args.data_server, affectors);

    loop {
        let encoded_msg = usb.recv_msgs().await;
        server_client
            .check_send_encoded(&encoded_msg)
            .await
            .wrap_err("Should be correctly encoded")
            .suggestion("Check if this needs to be updated")?;
    }
}

fn setup_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, Layer};

    let file_subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_ansi(false)
        .with_filter(tracing_subscriber::filter::EnvFilter::from_default_env());
    tracing_subscriber::registry()
        .with(file_subscriber)
        .with(ErrorLayer::default())
        .init();
}
