use std::{net::SocketAddr, time::Duration};

use clap::Parser;

use postcard_rpc::{
    host_client::{HostClient, IoClosed, Subscription},
    standard_icd::{WireError, ERROR_PATH},
};

use data_server::api::data_source;
use schema::ProtocolMsg;
use schema::ProtocolTopic;
use tokio::time::sleep;

#[derive(Parser)]
#[command(name = "local sensors")]
#[command(version = "1.0")]
#[command(about = "reads sensors attached to rpi gpio pins and i2c perhipheral")]
struct Cli {
    /// Where to send the data on the local system
    #[arg(short, long("data-server"))]
    data_server: SocketAddr,
    /// Serial number of the device to connect case insensitive
    #[arg(short, long)]
    serial_number: String,
}

struct ReconnectingUsbSub {
    serial_number: String,
    conn: Option<HostClient<WireError>>,
    sub: Option<Subscription<ProtocolMsg>>,
    logger: ratelimited_logger::RateLimitedLogger,
}

impl ReconnectingUsbSub {
    fn new(serial_number: String) -> Self {
        ReconnectingUsbSub {
            serial_number,
            conn: None,
            sub: None,
            logger: ratelimited_logger::RateLimitedLogger::new(),
        }
    }

    async fn recv(&mut self) -> ProtocolMsg {
        loop {
            if let Some(sub) = &mut self.sub {
                match sub.recv().await {
                    Some(msg) => return msg,
                    None => self.sub = None,
                }
            }

            if let Some(conn) = &mut self.conn {
                match conn.subscribe::<ProtocolTopic>(8).await {
                    Ok(sub) => {
                        self.sub = Some(sub);
                        continue;
                    }
                    Err(IoClosed) => self.conn = None,
                }
            }

            match HostClient::try_new_raw_nusb(
                |d| {
                    d.serial_number()
                        .is_some_and(|d| d.eq_ignore_ascii_case(&self.serial_number))
                },
                ERROR_PATH,
                1,
            ) {
                Ok(conn) => self.conn = Some(conn),
                Err(e) => {
                    let logger = &mut self.logger;
                    ratelimited_logger::warn!(logger; "Could not connect to usbdevice: {e:?}");
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    setup_tracing();
    let args = Cli::parse();

    let mut usb = ReconnectingUsbSub::new(args.serial_number);
    let mut server_client = data_source::reconnecting::Client::new(args.data_server, Vec::new());

    loop {
        let msg = usb.recv().await;
        let encoded_protocol_msg = msg.0;
        server_client
            .check_send_encoded(&encoded_protocol_msg)
            .await
            .expect("Should be correctly encoded");
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
