use std::vec;
use std::{net::SocketAddr, time::Duration};

use clap::Parser;

use color_eyre::eyre::{bail, eyre, Context};
use color_eyre::Section;
use data_server::api::data_source::reconnecting;
use nusb::transfer;
use protocol::Affector;
use tokio::time::{sleep, sleep_until, Instant};

use ratelimited_logger as rl;

#[derive(Parser)]
#[command(name = "local sensors")]
#[command(version = "1.0")]
#[command(about = "reads sensors attached to rpi gpio pins and i2c perhipheral")]
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

fn request_msg() -> transfer::ControlIn {
    transfer::ControlIn {
        control_type: transfer::ControlType::Vendor,
        recipient: transfer::Recipient::Interface,
        request: 42,
        value: 0,
        index: 0,
        // must match SEND_BUFFER_SIZE in usb nodes
        length: 208,
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

    async fn wait_for_affectors(&mut self) -> color_eyre::Result<Vec<Affector>> {
        let msg = self.recv().await;
        let msg = protocol::Msg::<50>::decode(msg.to_vec()).wrap_err(
            "Could not decode protocol::Msg, maybe the protocol library \
            has changed since this was compiled",
        )?;
        let protocol::Msg::AffectorList(list) = msg else {
            bail!(
                "Every usb node should send an affector list on connect,\
                this node did not"
            );
        };

        Ok(list.values.to_vec())
    }

    async fn recv(&mut self) -> Vec<u8> {
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
                match self.try_recv_step().await {
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

    async fn try_recv_step(&mut self) -> color_eyre::Result<Option<Vec<u8>>> {
        if let Some(device) = self.conn.take() {
            sleep_until(self.next_poll).await;
            self.next_poll = Instant::now() + Duration::from_millis(100);
            let msg = device
                .control_in(request_msg())
                .await
                .into_result()
                .wrap_err("Something went wrong with control_in request")?;

            self.conn = Some(device);
            return Ok(Some(msg));
        }

        let list: Vec<_> = nusb::list_devices()
            .wrap_err("Could not list usb devices")?
            .filter(|d| {
                d.serial_number()
                    .is_some_and(|d| d.eq_ignore_ascii_case(&self.serial_number))
            })
            .collect();

        self.conn = match list.as_slice() {
            [dev] => dev,
            [] => bail!("No usb device found with the correct serial"),
            more => {
                return Err(eyre!("Multiple usb devices have the same serial number")
                    .with_note(|| format!("they are: {more:?}")))
            }
        }
        .open()
        .map(Option::Some)
        .wrap_err("Could not open the usb device")
        .suggestion("Try running as sudo")
        .with_suggestion(|| {
            format!(
                "Add a .rules file in /etc/udev/rules.d with line: \
                ATTRS{{serial}}==\"{}\", MODE=\"660\", GROUP=\"{}\", TAG+=\"uaccess\"",
                self.serial_number,
                users::get_current_groupname()
                    .expect("process should always run as a group")
                    .to_string_lossy()
            )
        })?;

        Ok(None) // no errors but not done yet, call us again
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install().unwrap();
    setup_tracing();
    let args = Cli::parse();

    let mut usb = ReconnectingUsbSub::new(args.serial_number);
    let affectors = usb.wait_for_affectors().await?;
    let mut server_client = reconnecting::Client::new(args.data_server, affectors);

    loop {
        let encoded_msg = usb.recv().await;
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
