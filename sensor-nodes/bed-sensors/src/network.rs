use defmt::{debug, info, unwrap, warn};
use embassy_futures::select::{self, select};
use embassy_net::driver::Driver;
use embassy_net::tcp::{TcpReader, TcpSocket, TcpWriter};
use embassy_net::{Ipv4Address, Stack};
use embassy_time::{with_timeout, Duration, Instant, Timer};
use embedded_io_async::Write;
use protocol::large_bedroom::{self, bed};
use protocol::{affector, Affector, ErrorReport, SensorMessage};

use crate::channel::{PriorityValue, QueueItem, Queues};
use crate::rgb_led::LedHandle;

type SensMsg = SensorMessage<10>;

async fn collect_pending(publish: &Queues, reading: PriorityValue) -> SensorMessage<10> {
    let mut msg = SensMsg::default();
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

pub async fn handle(stack: &Stack<impl Driver>, publish: &Queues, led: LedHandle<'_>) {
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; max(SensMsg::ENCODED_SIZE, ErrorReport::ENCODED_SIZE) * 2];

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
        // Prevent out-dated data from being send
        publish.clear().await;

        let (reader, writer) = socket.split();
        match select(send_messages(writer, publish), receive_orders(reader, &led)).await {
            select::Either::First(e) => warn!("Error while sending messages: {}", e),
            select::Either::Second(e) => warn!("Error receiving orders: {}", e),
        };
        // Or the socket will hang for a while waiting to close this makes sure
        // we can reconnect instantly
        socket.abort();
        // Do not trigger data-server rate-limit
        Timer::after(Duration::from_secs(2)).await;
    }
}

fn affector_list() -> affector::ListMessage<5> {
    let mut list = affector::ListMessage::<5>::empty();
    unwrap!(list.values.push(Affector::LargeBedroom(
        protocol::large_bedroom::Affector::Bed(bed::Affector::RgbLed {
            red: 0,
            green: 0,
            blue: 0,
        }),
    )));
    unwrap!(list.values.push(Affector::LargeBedroom(
        protocol::large_bedroom::Affector::Bed(bed::Affector::Sps30FanClean),
    )));
    unwrap!(list.values.push(Affector::LargeBedroom(
        protocol::large_bedroom::Affector::Bed(bed::Affector::MhzZeroPointCalib),
    )));
    unwrap!(list.values.push(Affector::LargeBedroom(
        protocol::large_bedroom::Affector::Bed(bed::Affector::Nau7802LeftCalib),
    )));
    unwrap!(list.values.push(Affector::LargeBedroom(
        protocol::large_bedroom::Affector::Bed(bed::Affector::Nau7802RightCalib),
    )));

    list
}

async fn send_messages(mut tcp: TcpWriter<'_>, publish: &Queues) -> embassy_net::tcp::Error {
    let mut buf = [0; 1 + max(
        max(SensMsg::ENCODED_SIZE, ErrorReport::ENCODED_SIZE),
        affector::ListMessage::<5>::ENCODED_SIZE,
    )];
    let encoded_len = affector_list().encode_slice(&mut buf[1..]).len();
    buf[0] = protocol::Msg::<0>::AFFECTOR_LIST;
    let to_send = &buf[..=encoded_len];
    if let Err(e) = tcp.write_all(to_send).await {
        return e;
    }

    loop {
        let to_send = get_messages(publish, &mut buf).await;
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

async fn receive_orders(mut tcp: TcpReader<'_>, led: &LedHandle<'_>) -> ReadError {
    defmt::debug!("ready to receive orders");
    let mut decoder = affector::Decoder::default();
    let mut buf = [0u8; 100];
    loop {
        let res = tcp.read(&mut buf).await;
        let n_read = match res {
            Ok(0) => return ReadError::ConnectionClosed,
            Ok(n_read) => n_read,
            Err(e) => return ReadError::TcpError(e),
        };
        let mut read = &buf[..n_read];
        while let Some((item, remaining)) = decoder.feed(read) {
            read = remaining;

            #[allow(irrefutable_let_patterns)] // Will change in the future
            let Affector::LargeBedroom(large_bedroom::Affector::Bed(affector)) = item
            else {
                defmt::warn!("Got affector for other node");
                continue;
            };

            match affector {
                bed::Affector::MhzZeroPointCalib
                | bed::Affector::Nau7802LeftCalib
                | bed::Affector::Nau7802RightCalib
                | bed::Affector::Sps30FanClean => {
                    defmt::warn!("unimplemented affector: {:?}", affector)
                }
                bed::Affector::RgbLed { red, green, blue } => {
                    led.set_color(
                        red as f32 / u8::MAX as f32,
                        green as f32 / u8::MAX as f32,
                        blue as f32 / u8::MAX as f32,
                    )
                    .await
                }
            }
        }
    }
}
