use std::future::pending;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use color_eyre::Result;
use data_server::api::data_source;
use data_server::api::subscriber::{Client, SubMessage};
use data_server::server::{self, AffectorRegistar};
use protocol::large_bedroom::bed;
use protocol::Reading;
use protocol::{large_bedroom, Affector};
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

const TEST_READING: Reading = Reading::LargeBedroom(
    large_bedroom::Reading::Bed(bed::Reading::NumberPm2_5(0.0)),
);
const TEST_AFFECTOR: Affector = Affector::LargeBedroom(
    large_bedroom::Affector::Bed(bed::Affector::ResetNode),
);

async fn run_server(
    client_addr: impl Into<SocketAddr>,
    data_port: impl Into<SocketAddr>,
) -> Result<Done> {
    let (tx, rx) = mpsc::channel(2000);
    let affectors = AffectorRegistar::default();
    select! {
        e = server::client::handle(client_addr.into(), tx.clone(), affectors.clone()) => e.unwrap(),
        e = server::handle_nodes(data_port.into(), &tx, affectors) => e.unwrap(),
        e = server::spread_updates(rx) => e?,
    };

    Ok(Done::RunServer)
}

async fn send_sensor_value(data_port: u16) -> Result<Done> {
    tokio::time::sleep(Duration::from_millis(400)).await;

    let mut list = protocol::affector::ListMessage::<50>::empty();
    list.values
        .push(protocol::Affector::LargeBedroom(
            large_bedroom::Affector::Bed(bed::Affector::Sps30FanClean),
        ))
        .unwrap();
    let handshake = protocol::Msg::AffectorList(list).encode();

    let mut conn = TcpStream::connect(("127.0.0.1", data_port)).await.unwrap();
    conn.write_all(&handshake).await.unwrap();

    let mut sensor_msg = protocol::SensorMessage::<50>::default();
    sensor_msg.values.push(TEST_READING).unwrap();
    let sensor_msg = protocol::Msg::Readings(sensor_msg).encode();
    conn.write_all(&sensor_msg).await.unwrap();

    sleep(Duration::from_secs(999)).await;
    Ok(Done::SendValue)
}

async fn subscribe_and_receive_inner(sub_port: u16) -> Result<Done> {
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

async fn list_affectors_inner(sub_port: u16) -> Result<Done> {
    tokio::time::sleep(Duration::from_millis(500)).await;
    let list = Client::connect(
        (Ipv4Addr::LOCALHOST, sub_port),
        "api_integration_tests".to_owned(),
    )
    .await
    .unwrap()
    .list_affectors()
    .await
    .unwrap();

    assert_eq!(
        list,
        [protocol::Affector::LargeBedroom(
            large_bedroom::Affector::Bed(bed::Affector::Sps30FanClean)
        )]
    );

    Ok(Done::Test)
}

#[tokio::test]
async fn subscribe_and_receive() {
    logger::tracing::setup_for_tests();

    let sub_port = reserve_port::ReservedPort::random().unwrap();
    let data_port = reserve_port::ReservedPort::random().unwrap();
    let res = select! {
        e = run_server(([127,0,0,1], sub_port.port()), ([127,0,0,1], data_port.port())) => e,
        e = send_sensor_value(data_port.port()) => e,
        e = subscribe_and_receive_inner(sub_port.port()) => e,
    };
    assert_eq!(res.unwrap(), Done::Test);
}

#[tokio::test]
async fn list_affectors() {
    logger::tracing::setup_for_tests();

    let sub_port = reserve_port::ReservedPort::random().unwrap();
    let data_port = reserve_port::ReservedPort::random().unwrap();
    let res = select! {
        e = run_server(([127,0,0,1], sub_port.port()), ([127,0,0,1], data_port.port())) => e,
        e = send_sensor_value(data_port.port()) => e,
        e = list_affectors_inner(sub_port.port()) => e,
    };
    assert_eq!(res.unwrap(), Done::Test);
}

async fn trigger_affector(sub_port: u16) -> Result<Done> {
    tokio::time::sleep(Duration::from_millis(100)).await;
    let mut client = Client::connect(
        (Ipv4Addr::LOCALHOST, sub_port),
        "api_integration_tests".to_owned(),
    )
    .await
    .unwrap();

    client.actuate_affector(TEST_AFFECTOR).await.unwrap();
    Ok(pending::<Done>().await)
}

async fn recv_affector_order(data_port: u16) -> Result<Done> {
    let (tx, mut rx) = mpsc::channel(2);
    let _client = data_source::reconnecting::Client::new(
        (Ipv4Addr::LOCALHOST, data_port),
        vec![TEST_AFFECTOR],
        Some(tx),
    )
    .await
    .unwrap();

    let affector_order =
        tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .unwrap()
            .unwrap();

    assert_eq!(affector_order, TEST_AFFECTOR);

    Ok(Done::Test)
}

#[tokio::test]
async fn recv_affector() {
    logger::tracing::setup_for_tests();

    let sub_port = reserve_port::ReservedPort::random().unwrap();
    let data_port = reserve_port::ReservedPort::random().unwrap();
    let res = select! {
        e = run_server(([127,0,0,1], sub_port.port()), ([127,0,0,1], data_port.port())) => e,
        e = recv_affector_order(data_port.port()) => e,
        e = trigger_affector(sub_port.port()) => e,
    };
    assert_eq!(res.unwrap(), Done::Test);
}
