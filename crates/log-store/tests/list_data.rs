use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Once;
use std::time::Duration;

use futures::FutureExt;
use futures_concurrency::future::Race;
use protocol::large_bedroom::bed;
use protocol::{large_bedroom, Reading};
use temp_dir::TempDir;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Notify};
use tokio::time::sleep;
use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const fn test_readings(v: f32) -> [Reading; 2] {
    [
        Reading::LargeBedroom(large_bedroom::Reading::Bed(bed::Reading::Temperature(v))),
        Reading::LargeBedroom(large_bedroom::Reading::Bed(bed::Reading::Humidity(v))),
    ]
}

fn test_error() -> protocol::Error {
    protocol::Error::LargeBedroom(large_bedroom::Error::Bed(bed::Error::Setup(
        bed::SensorError::Sht31(
            heapless::String::from_str("log server integration test error").unwrap(),
        ),
    )))
}

async fn data_server(sub_port: impl Into<SocketAddr>, data_port: impl Into<SocketAddr>) {
    use data_server::server;

    let (tx, rx) = mpsc::channel(2000);
    tokio::select! {
        e = server::register_subs(sub_port.into(), &tx) => e.unwrap(),
        e = server::handle_data_sources(data_port.into(), &tx) => e.unwrap(),
        e = server::spread_updates(rx) => e.unwrap(),
    };
}

async fn send_sensor_values(data_port: u16, data_send: &Notify) {
    let mut conn = TcpStream::connect(("127.0.0.1", data_port)).await.unwrap();
    for v in [0.1, 0.2, 0.3] {
        let mut sensor_msg = protocol::SensorMessage::<50>::default();
        for val in test_readings(v) {
            sensor_msg.values.push(val).unwrap();
        }
        let encoded = protocol::Msg::Readings(sensor_msg).encode();
        conn.write_all(&encoded).await.unwrap();
        sleep(Duration::from_secs_f32(1.1)).await;
    }
    data_send.notify_waiters();
    sleep(Duration::from_secs(999)).await;
}

async fn send_sensor_errors(data_port: u16, data_send: &Notify) {
    let mut conn = TcpStream::connect(("127.0.0.1", data_port)).await.unwrap();
    for _ in [1, 2, 3, 4] {
        let report = protocol::ErrorReport::new(test_error());
        let encoded = protocol::Msg::<50>::ErrorReport(report).encode();
        conn.write_all(&encoded).await.unwrap();
        sleep(Duration::from_secs_f32(1.1)).await;
    }
    data_send.notify_waiters();
    sleep(Duration::from_secs(999)).await;
}

async fn check_client_get_percentiles(data_store_addr: SocketAddr, data_send: &Notify) {
    data_send.notified().await;
    sleep(Duration::from_secs_f32(0.1)).await;

    let mut client =
        log_store::api::Client::connect(data_store_addr, "data_store_example".to_owned())
            .await
            .unwrap();
    let test_device = test_readings(0.0).first().unwrap().device();
    let list = client.get_percentiles(test_device).await.unwrap();

    assert!(
        !list.is_empty(),
        "list is empty, should contain one reading"
    );
}

async fn check_client_get_logs(data_store_addr: SocketAddr, data_send: &Notify) {
    data_send.notified().await;
    sleep(Duration::from_secs_f32(0.1)).await;
    let mut client =
        log_store::api::Client::connect(data_store_addr, "data_store_example".to_owned())
            .await
            .unwrap();

    let test_device = test_readings(0.0).first().unwrap().device();
    let logs = client.get_logs(test_device).await.unwrap();

    assert_eq!(logs.len(), 4);
}

static SETUP_REPORTING: Once = Once::new();

fn setup_reporting() {
    SETUP_REPORTING.call_once(|| {
        color_eyre::install().unwrap();
        tracing_subscriber::registry()
            .with(ErrorLayer::default())
            .with(tracing_subscriber::fmt::layer().pretty().with_test_writer())
            .init();
    });
}

#[tokio::test]
async fn get_logs() {
    const DATA_SERVER_STARTUP: Duration = Duration::from_millis(20);
    const DATA_STORE_STARTUP: Duration = Duration::from_millis(20);
    const FIRST_MSG_PROCESSED: Duration = Duration::from_millis(1000);

    setup_reporting();

    let test_dir = TempDir::new().unwrap();

    let sub_port = reserve_port::ReservedPort::random().unwrap();
    let data_port = reserve_port::ReservedPort::random().unwrap();
    let store_port = reserve_port::ReservedPort::random().unwrap();

    let data_server_addr = SocketAddr::from(([127, 0, 0, 1], sub_port.port()));
    let data_store_addr = SocketAddr::from(([127, 0, 0, 1], store_port.port()));

    let errors_send = Notify::new();
    let run_data_server = data_server(
        ([127, 0, 0, 1], sub_port.port()),
        ([127, 0, 0, 1], data_port.port()),
    );
    let run_data_store = sleep(DATA_SERVER_STARTUP).then(|()| {
        log_store::server::run(data_server_addr, data_store_addr.port(), test_dir.path())
    });
    let send_sensor_value = sleep(DATA_SERVER_STARTUP + DATA_STORE_STARTUP)
        .then(|()| send_sensor_errors(data_port.port(), &errors_send));
    let run_test = sleep(DATA_SERVER_STARTUP + DATA_STORE_STARTUP + FIRST_MSG_PROCESSED)
        .then(|()| check_client_get_logs(data_store_addr, &errors_send));

    let res = (
        run_test.map(Result::Ok),
        send_sensor_value.map(Result::Ok),
        run_data_store,
        run_data_server.map(Result::Ok),
    )
        .race()
        .await;

    res.unwrap();
}

#[tokio::test]
async fn get_percentiles() {
    const DATA_SERVER_STARTUP: Duration = Duration::from_millis(20);
    const DATA_STORE_STARTUP: Duration = Duration::from_millis(20);
    const FIRST_MSG_PROCESSED: Duration = Duration::from_millis(1000);

    let test_dir = TempDir::new().unwrap();
    std::env::set_current_dir(test_dir.path()).unwrap();

    setup_reporting();

    let sub_port = reserve_port::ReservedPort::random().unwrap();
    let data_port = reserve_port::ReservedPort::random().unwrap();
    let store_port = reserve_port::ReservedPort::random().unwrap();

    let data_server_addr = SocketAddr::from(([127, 0, 0, 1], sub_port.port()));
    let data_store_addr = SocketAddr::from(([127, 0, 0, 1], store_port.port()));

    let data_send = Notify::new();
    let run_data_server = data_server(
        ([127, 0, 0, 1], sub_port.port()),
        ([127, 0, 0, 1], data_port.port()),
    );
    let run_data_store = sleep(DATA_SERVER_STARTUP).then(|()| {
        log_store::server::run(data_server_addr, data_store_addr.port(), test_dir.path())
    });
    let send_sensor_value = sleep(DATA_SERVER_STARTUP + DATA_STORE_STARTUP)
        .then(|()| send_sensor_values(data_port.port(), &data_send));
    let run_test = sleep(DATA_SERVER_STARTUP + DATA_STORE_STARTUP + FIRST_MSG_PROCESSED)
        .then(|()| check_client_get_percentiles(data_store_addr, &data_send));

    let res = (
        run_test.map(Result::Ok),
        send_sensor_value.map(Result::Ok),
        run_data_store,
        run_data_server.map(Result::Ok),
    )
        .race()
        .await;

    res.unwrap();
}
