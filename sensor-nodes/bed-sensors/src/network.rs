use defmt::{info, unwrap, warn};
use embassy_net::driver::Driver;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv4Address, Stack};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{with_timeout, Duration, Instant, Timer};
use embedded_io_async::Write;
use protocol::SensorMessage;

use crate::channel::Channel;

type Msg = SensorMessage<4>;

async fn get_messages(publish: &Channel, msg: &mut Msg) {
    msg.values.clear();
    let next = publish.receive().await;
    let low_priority = next.low_priority();
    unwrap!(msg.values.push(next.value));

    if low_priority {
        let deadline = Instant::now() + Duration::from_millis(200);
        while msg.space_left() {
            let until = deadline.saturating_duration_since(Instant::now());
            match with_timeout(until, publish.receive()).await {
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
}

pub async fn send_published(
    stack: &Stack<impl Driver>,
    publish: &Channel,
    network_up: &Signal<NoopRawMutex, ()>,
) {
    let mut rx_buffer = [0; 800];
    let mut tx_buffer = [0; Msg::ENCODED_SIZE * 2];

    let mut msg = Msg::new();
    let mut encoded_msg_buffer = [0; Msg::ENCODED_SIZE];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(5)));
    let host_addr = Ipv4Address::new(192, 168, 1, 43);
    let host_port = 1234;

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
                network_up.signal(());
            }
        }

        get_messages(publish, &mut msg).await;
        let to_send = msg.encode_slice(&mut encoded_msg_buffer);

        if let Err(e) = socket.write_all(to_send).await {
            warn!("write error: {:?}", e);
        }
    }
}
