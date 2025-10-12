use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use color_eyre::Result;
use data_server::api::subscriber::{Client, SubMessage};
use data_server::server::{self, AffectorRegistar};
use protocol::large_bedroom;
use protocol::large_bedroom::desk_right;
use protocol::{pir, Reading};
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

const TEST_PIR_ACTIVE: Reading =
    Reading::LargeBedroom(large_bedroom::Reading::DeskRight(
        desk_right::Reading::Pir(pir::Status::OngoingActivity),
    ));
const TEST_PIR_WENT_DARK: Reading =
    Reading::LargeBedroom(large_bedroom::Reading::DeskRight(
        desk_right::Reading::Pir(pir::Status::Unknown),
    ));

async fn run_server(
    client_addr: impl Into<SocketAddr>,
    data_port: impl Into<SocketAddr>,
) -> Result<Done> {
    let (tx, rx) = mpsc::channel(2000);
    let affectors = AffectorRegistar::default();
    select! {
        e = server::client::handle(client_addr.into(), tx.clone(), affectors.clone()) => e.unwrap(),
        e = server::handle_nodes(data_port.into(), &tx, affectors) => e.unwrap(),
        e = server::handle_updates(rx) => e?,
    };

    Ok(Done::RunServer)
}

async fn send_pir_active(data_port: u16) -> Result<Done> {
    tokio::time::sleep(Duration::from_millis(400)).await;

    let list = protocol::affector::ListMessage::<50>::empty();
    let handshake = protocol::Msg::AffectorList(list).encode();

    let mut conn = TcpStream::connect(("127.0.0.1", data_port)).await.unwrap();
    conn.write_all(&handshake).await.unwrap();

    let mut sensor_msg = protocol::SensorMessage::<50>::default();
    sensor_msg.values.push(TEST_PIR_ACTIVE).unwrap();
    let sensor_msg = protocol::Msg::Readings(sensor_msg).encode();
    conn.write_all(&sensor_msg).await.unwrap();

    sleep(Duration::from_secs(999)).await;
    Ok(Done::SendValue)
}

async fn wait_for_pir_then_unknown(sub_port: u16) -> Result<Done> {
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
    assert!(matches!(received, SubMessage::Reading(TEST_PIR_ACTIVE)));
    tokio::time::sleep(Duration::from_millis(5100)).await;
    let received = sub.next().await.unwrap();
    assert!(
        matches!(received, SubMessage::Reading(TEST_PIR_WENT_DARK)),
        "got: {received:?}"
    );

    Ok(Done::Test)
}

#[tokio::test]
async fn pir_unknown_send_if_active_pir_heartbeat_stops() {
    logger::tracing::setup_for_tests();

    let sub_port = reserve_port::ReservedPort::random().unwrap();
    let data_port = reserve_port::ReservedPort::random().unwrap();
    let res = select! {
        e = run_server(([127,0,0,1], sub_port.port()), ([127,0,0,1], data_port.port())) => e,
        e = send_pir_active(data_port.port()) => e,
        e = wait_for_pir_then_unknown(sub_port.port()) => e,
    };
    assert_eq!(res.unwrap(), Done::Test);
}
