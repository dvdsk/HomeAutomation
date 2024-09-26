use std::{net::SocketAddr, time::Duration};

use clap::Parser;

use color_eyre::eyre::{eyre, Context};
use color_eyre::Section;
use data_server::api::data_source;
use nusb::transfer;
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
    conn: Option<nusb::Device>,
    logger: ratelimited_logger::RateLimitedLogger,
}

fn request_msg() -> transfer::ControlIn {
    transfer::ControlIn {
        control_type: transfer::ControlType::Vendor,
        recipient: transfer::Recipient::Interface,
        request: 42,
        value: 0,
        index: 0,
        length: protocol::Msg::<10>::max_size()
            .try_into()
            .expect("should fit"),
    }
}

type EncodedProtocolMsg = Vec<u8>;

impl ReconnectingUsbSub {
    fn new(serial_number: String) -> Self {
        ReconnectingUsbSub {
            serial_number,
            conn: None,
            logger: ratelimited_logger::RateLimitedLogger::default(),
        }
    }

    async fn recv(&mut self) -> EncodedProtocolMsg {
        let mut retry_period = Duration::from_millis(100);
        loop {
            match self.try_recv_step().await {
                Ok(Some(msg)) => return msg,
                Ok(None) => (),
                Err(e) => {
                    let logger = &mut self.logger;
                    rl::warn!(logger; "{e:?}");
                    retry_period *= 2;
                    let retry_period = retry_period.min(Duration::from_secs(30));
                    sleep(retry_period).await;
                }
            }
        }
    }

    async fn try_recv_step(&mut self) -> color_eyre::Result<Option<EncodedProtocolMsg>> {
        if let Some(device) = self.conn.take() {
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
            [] => return Err(eyre!("No usb device found with the correct serial")),
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
async fn main() {
    setup_tracing();
    let args = Cli::parse();

    let mut usb = ReconnectingUsbSub::new(args.serial_number);
    let mut server_client = data_source::reconnecting::Client::new(args.data_server, Vec::new());

    loop {
        let last_poll = Instant::now();
        let encoded_msg = usb.recv().await;
        server_client
            .check_send_encoded(&encoded_msg)
            .await
            .expect("Should be correctly encoded");
        sleep_until(last_poll + Duration::from_millis(100)).await;
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
