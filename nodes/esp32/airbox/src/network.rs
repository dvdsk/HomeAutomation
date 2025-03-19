use defmt::{debug, info, unwrap, warn};
use embassy_futures::select::{self, select};
use embassy_net::tcp::{TcpReader, TcpSocket, TcpWriter};
use embassy_net::{Ipv4Address, Runner};
use embassy_time::{Duration, Timer};
use embedded_io_async::Write;
use esp_wifi::wifi::{
    ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent,
    WifiState,
};
use protocol::affector::DeserializeError;
use protocol::large_bedroom::airbox;
use protocol::{Affector, ErrorReport, affector};

use crate::fans::Fans;
use crate::{CONFIG, Queue};
type SensMsg = protocol::SensorMessage<10>;

const fn max(a: usize, b: usize) -> usize {
    [a, b][(a < b) as usize]
}

pub async fn handle(
    stack: &embassy_net::Stack<'_>,
    publish: &Queue,
    fans: &Fans,
) {
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer =
        [0; max(SensMsg::ENCODED_SIZE, protocol::ErrorReport::ENCODED_SIZE)
            * 2];

    let mut socket = TcpSocket::new(*stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(5)));
    socket.set_keep_alive(Some(Duration::from_secs(1)));
    let host_addr = Ipv4Address::new(192, 168, 1, 43);
    let host_port = 1234;

    debug!("Configured socket and connecting");
    loop {
        debug!("socket state: {:?}", socket.state());
        if let Err(e) = socket.connect((host_addr, host_port)).await {
            warn!("connect error: {}", e);
            Timer::after_secs(5).await;
            continue;
        }

        info!("(re-)connected");
        // Prevent out-dated data from being send
        publish.clear();

        let (reader, writer) = socket.split();
        match select(
            send_messages(writer, publish),
            receive_orders(reader, fans, publish),
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

async fn send_messages(
    mut tcp: TcpWriter<'_>,
    publish: &Queue,
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
        let to_send = match publish.receive().await {
            Ok(reading) => {
                let mut msg = SensMsg::default();
                defmt::unwrap!(msg.values.push(reading));
                let encoded_len = msg.encode_slice(&mut buf[1..]).len();
                buf[0] = protocol::Msg::<0>::READINGS;
                &buf[..=encoded_len]
            }
            Err(error) => {
                let encoded_len =
                    ErrorReport::new(error).encode_slice(&mut buf[1..]).len();
                buf[0] = protocol::Msg::<0>::ERROR_REPORT;
                &buf[..=encoded_len]
            }
        };

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
    fans: &Fans,
    publish: &Queue,
) -> ReadError {
    defmt::debug!("ready to receive orders");
    let mut decoder = protocol::affector::Decoder::default();
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

            let protocol::Affector::LargeBedroom(
                protocol::large_bedroom::Affector::Airbox(affector),
            ) = item
            else {
                defmt::error!("Got affector for other node");
                continue;
            };

            defmt::info!("got affector order: {:?}", affector);
            match affector {
                protocol::large_bedroom::airbox::Affector::FanPower {
                    power,
                } => {
                    if let Err(e) = fans.set_power(power).await {
                        publish.send(Err(e)).await;
                    }
                }
                protocol::large_bedroom::airbox::Affector::ResetNode => todo!(),
            }
        }
    }
}

#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
pub async fn connection(mut controller: WifiController<'static>) {
    defmt::info!("start connection task");
    loop {
        if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
            // Wait until we're no longer connected
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                // The constant `CONFIG` is auto-generated by `toml_config`.
                ssid: CONFIG.wifi_ssid.try_into().unwrap(),
                password: CONFIG.wifi_psk.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            defmt::info!("Starting wifi");
            controller.start_async().await.unwrap();
            defmt::info!("Wifi started!");
        }
        defmt::info!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => defmt::info!("Wifi connected!"),
            Err(e) => {
                defmt::info!("Failed to connect to wifi: {}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

fn affector_list() -> affector::ListMessage<2> {
    let mut list = affector::ListMessage::<2>::empty();
    unwrap!(list.values.push(Affector::LargeBedroom(
        protocol::large_bedroom::Affector::Airbox(airbox::Affector::FanPower {
            power: 0,
        }),
    )));
    unwrap!(list.values.push(Affector::LargeBedroom(
        protocol::large_bedroom::Affector::Airbox(airbox::Affector::ResetNode)
    )));

    list
}
