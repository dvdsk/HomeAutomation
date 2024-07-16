use std::net::SocketAddr;
use std::time::Duration;

use color_eyre::Result;
use data_server::server;
use data_server::subscriber::SubMessage;
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
    sub_port: impl Into<SocketAddr>,
    data_port: impl Into<SocketAddr>,
) -> Result<Done> {
    let (tx, rx) = mpsc::channel(2000);
    select! {
        e = server::register_subs(sub_port.into(), &tx) => e?,
        e = server::handle_data_sources(data_port.into(), &tx) => e?,
        e = server::spread_updates(rx) => e?,
    };

    Ok(Done::RunServer)
}

const TEST_READING: Reading =
    Reading::LargeBedroom(large_bedroom::Reading::Bed(bed::Reading::Temperature(0.0)));

async fn send_sensor_value(data_port: u16) -> Result<Done> {
    tokio::time::sleep(Duration::from_millis(100)).await;
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
    let mut sub =
        data_server::AsyncSubscriber::connect(("127.0.0.1", sub_port), "api_integration_tests")
            .await
            .unwrap();
    let received = sub.next_msg().await.unwrap();
    dbg!(&received);
    assert!(matches!(received, SubMessage::Reading(TEST_READING)));

    Ok(Done::Test)
}

#[tokio::test]
async fn main() {
    let sub_port = reserve_port::ReservedPort::random().unwrap();
    let data_port = reserve_port::ReservedPort::random().unwrap();
    let res = select! {
        e = run_server(([127,0,0,1], sub_port.port()), ([127,0,0,1], data_port.port())) => e,
        e = send_sensor_value(data_port.port()) => e,
        e = subscribe_and_receive(sub_port.port()) => e,
    };
    assert_eq!(res.unwrap(), Done::Test);
}
