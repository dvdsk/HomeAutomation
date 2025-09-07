use defmt::{debug, info, unwrap, warn};
use embassy_executor::task;
use embassy_futures::select::{self, select};
use embassy_net::tcp::{TcpReader, TcpSocket, TcpWriter};
use embassy_net::{Ipv4Address, Stack};
use embassy_time::{with_timeout, Duration, Instant, Timer};
use embedded_io_async::Write;
use protocol::affector::DeserializeError;
use protocol::large_bedroom::{self, bed};
use protocol::{affector, Affector, ErrorReport, SensorMessage};

use crate::channel::{PriorityValue, QueueItem, Queues};
use crate::rgb_led::LedHandle;
use crate::sensors::slow;

type SensMsg = SensorMessage<10>;

async fn collect_pending(
    publish: &Queues,
    reading: PriorityValue,
) -> SensorMessage<10> {
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
            let encoded_len =
                ErrorReport::new(error).encode_slice(&mut buf[1..]).len();
            buf[0] = protocol::Msg::<0>::ERROR_REPORT;
            &buf[..=encoded_len]
        }
    }
}

const fn max(a: usize, b: usize) -> usize {
    [a, b][(a < b) as usize]
}

#[task]
pub async fn handle(
    stack: Stack<'static>,
    publish: &'static Queues,
    led: LedHandle,
    driver_orderers: &'static slow::DriverOrderers,
) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; max(
        max(SensMsg::ENCODED_SIZE, ErrorReport::ENCODED_SIZE) * 2,
        4096,
    )];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(5)));
    socket.set_keep_alive(Some(Duration::from_secs(1)));
    let host_addr = Ipv4Address::new(192, 168, 1, 43);
    let host_port = 1234;

    let mut sequential_errors = 0;

    debug!("Configured socket and connecting");
    loop {
        debug!("socket state: {:?}", socket.state());
        if let Err(e) = socket.connect((host_addr, host_port)).await {
            if sequential_errors > 3 {
                defmt::info!("failing to reconnect, resetting node");
                defmt::flush();
                Timer::after_secs(1).await;
                cortex_m::peripheral::SCB::sys_reset();
            }
            warn!("connect error: {}", e);
            sequential_errors += 1;
            Timer::after_secs(5).await;
            continue;
        }

        info!("(re-)connected");
        sequential_errors = 0;
        // Prevent out-dated data from being send
        publish.clear().await;

        let (reader, writer) = socket.split();
        match select(
            send_messages(writer, publish),
            receive_orders(reader, &led, driver_orderers),
        )
        .await
        {
            select::Either::First(e) => {
                warn!("Error while sending messages: {}", e)
            }
            select::Either::Second(e) => warn!("Error receiving orders: {}", e),
        };
        // Or the socket will hang for a while waiting to close this makes sure
        // we can reconnect instantly
        socket.abort();
        Timer::after_secs(60).await; // Experiment: does this help?
    }
}

fn affector_list() -> affector::ListMessage<6> {
    let mut list = affector::ListMessage::<6>::empty();
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
        protocol::large_bedroom::Affector::Bed(
            bed::Affector::MhzZeroPointCalib
        ),
    )));
    unwrap!(list.values.push(Affector::LargeBedroom(
        protocol::large_bedroom::Affector::Bed(bed::Affector::Nau7802LeftCalib),
    )));
    unwrap!(list.values.push(Affector::LargeBedroom(
        protocol::large_bedroom::Affector::Bed(
            bed::Affector::Nau7802RightCalib
        ),
    )));
    unwrap!(list.values.push(Affector::LargeBedroom(
        protocol::large_bedroom::Affector::Bed(bed::Affector::ResetNode)
    )));

    list
}

async fn send_messages(
    mut tcp: TcpWriter<'_>,
    publish: &Queues,
) -> embassy_net::tcp::Error {
    let mut buf = [0; 1 + max(
        max(SensMsg::ENCODED_SIZE, ErrorReport::ENCODED_SIZE),
        affector::ListMessage::<5>::ENCODED_SIZE,
    )];
    let encoded_len = affector_list().encode_slice(&mut buf[1..]).len();
    buf[0] = protocol::Msg::<5>::AFFECTOR_LIST;
    let to_send = &buf[..=encoded_len];
    if let Err(e) = tcp.write_all(to_send).await {
        return e;
    }
    if let Err(e) = tcp.flush().await {
        return e;
    }

    loop {
        let to_send = get_messages(publish, &mut buf).await;
        if let Err(e) = tcp.write_all(to_send).await {
            return e;
        }
        if let Err(e) = tcp.flush().await {
            return e;
        }
    }
}

#[derive(defmt::Format)]
enum ReadError {
    ConnectionClosed,
    TcpError(embassy_net::tcp::Error),
    Deserialize(DeserializeError),
}

async fn receive_orders(
    mut tcp: TcpReader<'_>,
    led: &LedHandle,
    driver_orderers: &slow::DriverOrderers,
) -> ReadError {
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

        loop {
            let (item, remaining) = match decoder.feed(read) {
                Ok(Some((item, remaining))) => (item, remaining),
                Ok(None) => break,
                Err(e) => return ReadError::Deserialize(e),
            };

            read = remaining;

            let Affector::LargeBedroom(large_bedroom::Affector::Bed(affector)) =
                item
            else {
                defmt::error!("Got affector for other node");
                continue;
            };

            defmt::info!("got affector order: {:?}", affector);
            match affector {
                bed::Affector::Nau7802LeftCalib
                | bed::Affector::Nau7802RightCalib => {
                    defmt::warn!("unimplemented affector: {:?}", affector)
                }
                bed::Affector::MhzZeroPointCalib => {
                    driver_orderers.mhz.send(()).await;
                }
                bed::Affector::Sps30FanClean => {
                    driver_orderers.sps.send(()).await;
                }
                bed::Affector::RgbLed { red, green, blue } => {
                    led.set_color(
                        red as f32 / u8::MAX as f32,
                        green as f32 / u8::MAX as f32,
                        blue as f32 / u8::MAX as f32,
                    )
                    .await
                }
                bed::Affector::ResetNode => {
                    defmt::info!("resetting node as orderd via affector");
                    defmt::flush();
                    cortex_m::peripheral::SCB::sys_reset();
                }
            }
        }
    }
}
