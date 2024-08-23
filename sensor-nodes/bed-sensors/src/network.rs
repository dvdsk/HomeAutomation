use defmt::{debug, info, unwrap, warn};
use embassy_futures::select::{self, select};
use embassy_net::driver::Driver;
use embassy_net::tcp::{TcpReader, TcpSocket, TcpWriter};
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

pub async fn handle(stack: &Stack<impl Driver>, publish: &Queues) {
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; max(Msg::ENCODED_SIZE, ErrorReport::ENCODED_SIZE) * 2];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(5)));
    socket.set_keep_alive(Some(Duration::from_secs(1)));
    let host_addr = Ipv4Address::new(192, 168, 1, 43);
    let host_port = 1234;

    debug!("Configured socket and connecting");
    loop {
        debug!("socket state: {:?}", socket.state());
        if let Err(e) = socket.connect((host_addr, host_port)).await {
            warn!("connect error: {}", e);
            Timer::after_secs(1).await;
            continue;
        }
        info!("(re-)connected");
        // prevent out-dated data from being send
        publish.clear().await;

        let (reader, writer) = socket.split();
        match select(send_messages(writer, publish), receive_orders(reader)).await {
            select::Either::First(e) => warn!("Error while sending messages: {}", e),
            select::Either::Second(e) => warn!("Error receiving orders: {}", e),
        };
        // or the socket will hang for a while waiting to close this makes sure
        // we can reconnect instantly
        socket.abort();
    }
}

async fn send_messages(mut tcp: TcpWriter<'_>, publish: &Queues) -> embassy_net::tcp::Error {
    let mut encoded_msg_buffer = [0; max(Msg::ENCODED_SIZE, ErrorReport::ENCODED_SIZE)];
    loop {
        let to_send = get_messages(publish, &mut encoded_msg_buffer).await;
        if let Err(e) = tcp.write_all(to_send).await {
            return e;
        }
    }
}

#[derive(defmt::Format)]
enum ReadError {
    ConnectionClosed,
    TcpError(embassy_net::tcp::Error),
}

async fn receive_orders(mut tcp: TcpReader<'_>) -> ReadError {
    let mut buf = [0u8; 100];
    loop {
        let res = tcp.read(&mut buf).await;
        let n_read = match res {
            Ok(0) => return ReadError::ConnectionClosed,
            Ok(n_read) => n_read,
            Err(e) => return ReadError::TcpError(e),
        };
        let mut read = &buf[..n_read];
        let mut decoder = protocol::affector::Decoder::default();
        while let Some((item, remaining)) = decoder.feed(read) {
            read = remaining;
            defmt::info!("received item: {:?}", item)
        }
    }
}
