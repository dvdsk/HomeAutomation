use defmt::{debug, error, info, unwrap, warn};
use embassy_net::driver::Driver;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv4Address, Stack};
use embassy_time::{with_timeout, Duration, Instant, Timer};
use embedded_io_async::Write;
use protocol::{ErrorReport, SensorMessage};

use crate::channel::{PriorityValue, QueueItem, Queues};

type Msg = SensorMessage<10>;

async fn collect_pending(publish: &Queues, reading: PriorityValue) -> SensorMessage<10> {
    let mut msg = Msg::default();
    let low_priority = reading.low_priority();
    unwrap!(msg.values.push(reading.value));

    if low_priority {
        let deadline = Instant::now() + Duration::from_millis(200);
        while msg.space_left() {
            let until = deadline.saturating_duration_since(Instant::now());
            match with_timeout(until, publish.receive_reading()).await {
                Ok(new) if new.low_priority() => {
                    unwrap!(msg.values.push(new.value));
                }
                Ok(new) => {
                    unwrap!(msg.values.push(new.value));
                    break;
                }
                Err(_timeout) => break,
            }
        }
    } else {
        while msg.space_left() {
            let Some(next) = publish.next_ready() else {
                break;
            };
            unwrap!(msg.values.push(next.value));
        }
    }
    msg
}

async fn get_messages<'a>(publish: &Queues, buf: &'a mut [u8]) -> &'a [u8] {
    let next = publish.receive().await;
    match next {
        QueueItem::Reading(reading) => {
            let msg = collect_pending(publish, reading).await;
            let encoded_len = msg.encode_slice(&mut buf[1..]).len();
            buf[0] = protocol::Msg::<0>::READINGS;
            &buf[..=encoded_len]
        }
        QueueItem::Error(error) => {
            let error = protocol::large_bedroom::Error::Bed(error.into());
            let error = protocol::Error::LargeBedroom(error);
            let encoded_len = ErrorReport::new(error).encode_slice(&mut buf[1..]).len();
            buf[0] = protocol::Msg::<0>::ERROR_REPORT;
            &buf[..=encoded_len]
        }
    }
}

const fn max(a: usize, b: usize) -> usize {
    [a, b][(a < b) as usize]
}

pub struct ConnDownTooLong;
pub async fn send_published(stack: &Stack<impl Driver>, publish: &Queues) -> ConnDownTooLong {
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; max(Msg::ENCODED_SIZE, ErrorReport::ENCODED_SIZE) * 2];
    let mut encoded_msg_buffer = [0; max(Msg::ENCODED_SIZE, ErrorReport::ENCODED_SIZE)];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(5)));
    let host_addr = Ipv4Address::new(192, 168, 1, 43);
    let host_port = 1234;

    // connection sometimes does not recover from going down
    // track how long it has been down and reset the chip if
    // it has been down too long.
    let mut since_last_connection: Option<Instant> = None;
    let mut connected = false;

    debug!("Configured socket and connecting");
    loop {
        if !connected {
            match since_last_connection {
                Some(went_down) if went_down.elapsed() > Duration::from_secs(20) => {
                    error!("connection down to long, might have gotten stuck: resetting");
                    return ConnDownTooLong;
                }
                Some(_) => (),
                None => {
                    since_last_connection = Some(Instant::now());
                    warn!("warning connection down, resetting if not up soon");
                }
            }
        }

        if !connected {
            if let Err(e) = socket.connect((host_addr, host_port)).await {
                warn!("connect error: {:?}", e);
                Timer::after_secs(1).await;
                continue;
            }
            info!("(re-)connected");
            connected = true;
            // prevent out-dated data from being send
            publish.clear().await;
        }

        let to_send = get_messages(publish, &mut encoded_msg_buffer).await;
        if let Err(e) = socket.write_all(to_send).await {
            warn!("write error: {:?}", e);
            connected = false;
        }
    }
}
