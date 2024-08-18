use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use color_eyre::Result;
use data_server::api::{Client, SubMessage};
use data_server::server;
use protocol::large_bedroom;
use protocol::large_bedroom::bed;
use protocol::Reading;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc;
use tokio::time::sleep;

#[derive(Debug, PartialEq)]
enum Done {
    RunServer,
    SendValue,
    Test,
}

async fn run_server(
    client_addr: impl Into<SocketAddr>,
    data_port: impl Into<SocketAddr>,
) -> Result<Done> {
    let (tx, rx) = mpsc::channel(2000);
    select! {
        e = server::client::handle(client_addr.into(), tx.clone()) => e?,
        e = server::handle_data_sources(data_port.into(), &tx) => e?,
        e = server::spread_updates(rx) => e?,
    };

    Ok(Done::RunServer)
}

const TEST_READING: Reading =
    Reading::LargeBedroom(large_bedroom::Reading::Bed(bed::Reading::Temperature(0.0)));

async fn send_sensor_value(data_port: u16) -> Result<Done> {
    tokio::time::sleep(Duration::from_millis(500)).await;
    let mut sensor_msg = protocol::SensorMessage::<50>::default();
    sensor_msg.values.push(TEST_READING).unwrap();
    let encoded = protocol::Msg::Readings(sensor_msg).encode();

    let mut conn = TcpStream::connect(("127.0.0.1", data_port)).await.unwrap();
    conn.write_all(&encoded).await.unwrap();
    sleep(Duration::from_secs(999)).await;
    Ok(Done::SendValue)
}

async fn subscribe_and_receive(sub_port: u16) -> Result<Done> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    let mut sub = Client::connect(
        (Ipv4Addr::LOCALHOST, sub_port),
        "api_integration_tests".to_owned(),
    )
    .await
    .unwrap()
    .subscribe()
    .await
    .unwrap();

    let received = sub.next().await.unwrap();
    assert!(matches!(received, SubMessage::Reading(TEST_READING)));

    Ok(Done::Test)
}

#[tokio::test]
async fn main() {
    setup_tracing();

    let sub_port = reserve_port::ReservedPort::random().unwrap();
    let data_port = reserve_port::ReservedPort::random().unwrap();
    let res = select! {
        e = run_server(([127,0,0,1], sub_port.port()), ([127,0,0,1], data_port.port())) => e,
        e = send_sensor_value(data_port.port()) => e,
        e = subscribe_and_receive(sub_port.port()) => e,
    };
    assert_eq!(res.unwrap(), Done::Test);
}

fn setup_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt, Layer};

    color_eyre::install().unwrap();

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
