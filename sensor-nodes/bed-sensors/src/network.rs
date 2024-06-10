use defmt::{debug, info, unwrap, warn};
use embassy_net::driver::Driver;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv4Address, Stack};
use embassy_time::{with_timeout, Duration, Instant, Timer};
use embedded_io_async::Write;
use protocol::{ErrorReport, SensorMessage};

use crate::channel::{QueueItem, PriorityValue, Queues};

type Msg = SensorMessage<10>;

async fn collect_pending(publish: &Queues, reading: PriorityValue) -> Msg {
    let mut msg = Msg::new();
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
            msg.encode_slice(buf)
        }
        QueueItem::Error(error) => {
            let error = protocol::large_bedroom::Error::Bed(error.into());
            let error = protocol::Error::LargeBedroom(error);
            ErrorReport::new(error).encode_slice(buf)
        }
    }
}

const fn max(a: usize, b: usize) -> usize {
    [a, b][(a < b) as usize]
}

pub async fn send_published(stack: &Stack<impl Driver>, publish: &Queues) {
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; max(Msg::ENCODED_SIZE, ErrorReport::ENCODED_SIZE) * 2];
    let mut encoded_msg_buffer = [0; max(Msg::ENCODED_SIZE, ErrorReport::ENCODED_SIZE)];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    // socket.set_timeout(Some(Duration::from_secs(5)));
    let host_addr = Ipv4Address::new(192, 168, 1, 46);
    let host_port = 1234;

    debug!("Configured socket and connecting");
    loop {
        let connected = socket.remote_endpoint().is_some();
        if !connected {
            if let Err(e) = socket.connect((host_addr, host_port)).await {
                warn!("connect error: {:?}", e);
                Timer::after_secs(1).await;
                continue;
            } else {
                info!("(re-)connected");
                // prevent out-dated data from being send
                publish.clear().await;
            }
        }

        let to_send = get_messages(publish, &mut encoded_msg_buffer).await;
        if let Err(e) = socket.write_all(to_send).await {
            warn!("write error: {:?}", e);
        }
    }
}
