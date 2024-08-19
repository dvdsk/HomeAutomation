use std::net::SocketAddr;
use std::sync::Once;
use std::time::Duration;

use data_server::server::AffectorRegistar;
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

async fn data_server(client_addr: impl Into<SocketAddr>, data_port: impl Into<SocketAddr>) {
    use data_server::server;

    let (tx, rx) = mpsc::channel(2000);
    let affectors = AffectorRegistar::default();
    tokio::select! {
        e = server::client::handle(client_addr.into(), tx.clone(), affectors.clone()) => e.unwrap(),
        e = server::handle_nodes(data_port.into(), &tx, affectors) => e.unwrap(),
        e = server::spread_updates(rx) => e.unwrap(),
    };
}

async fn send_sensor_values(data_port: u16, values: &[f32], data_send: &Notify) {
    let mut conn = TcpStream::connect(("127.0.0.1", data_port)).await.unwrap();
    for v in values {
        let mut sensor_msg = protocol::SensorMessage::<50>::default();
        for val in test_readings(*v) {
            sensor_msg.values.push(val).unwrap();
        }
        let encoded = protocol::Msg::Readings(sensor_msg).encode();
        conn.write_all(&encoded).await.unwrap();
        sleep(Duration::from_secs_f32(1.1)).await;
    }
    data_send.notify_waiters();
    sleep(Duration::from_secs(999)).await;
}

async fn check_client_list_data(data_store_addr: SocketAddr, data_send: &Notify) {
    data_send.notified().await;
    sleep(Duration::from_secs_f32(0.1)).await;

    let mut client =
        data_store::api::Client::connect(data_store_addr, "data_store_example".to_owned())
            .await
            .unwrap();
    let list = client.list_data().await.unwrap();

    assert!(
        !list.is_empty(),
        "list is empty, should contain one reading"
    );
    assert!(
        list.iter().any(|r| {
            if let protocol::Reading::LargeBedroom(protocol::large_bedroom::Reading::Bed(
                protocol::large_bedroom::bed::Reading::Temperature(_),
            )) = r
            {
                true
            } else {
                false
            }
        }),
        "list is missing expected reading bed::Reading::Temperature. \
        Full list is: {list:?}"
    )
}

async fn check_client_get_data(
    data_store_addr: SocketAddr,
    sensor_values: &[f32],
    data_send: &Notify,
) {
    data_send.notified().await;
    sleep(Duration::from_secs_f32(0.1)).await;
    let mut client =
        data_store::api::Client::connect(data_store_addr, "data_store_example".to_owned())
            .await
            .unwrap();
    let (time, data) = client
        .get_data(
            jiff::Timestamp::now() - jiff::Span::default().seconds(30),
            jiff::Timestamp::now() + jiff::Span::default().seconds(30),
            test_readings(0.0)[0].clone(),
            5,
        )
        .await
        .unwrap();

    assert_eq!(time.len(), data.len());
    assert!(data
        .into_iter()
        .zip(sensor_values.into_iter().copied())
        .inspect(|r| println!("(got, expected): {r:?}"))
        .all(|(a, b)| (a - b).abs() < 0.1))
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
async fn list_data() {
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

    let data_send = Notify::new();
    let run_data_server = data_server(
        ([127, 0, 0, 1], sub_port.port()),
        ([127, 0, 0, 1], data_port.port()),
    );
    let run_data_store = sleep(DATA_SERVER_STARTUP).then(|()| {
        data_store::server::run(data_server_addr, data_store_addr.port(), test_dir.path())
    });
    let send_sensor_value = sleep(DATA_SERVER_STARTUP + DATA_STORE_STARTUP)
        .then(|()| send_sensor_values(data_port.port(), &[0.0], &data_send));
    let run_test = sleep(DATA_SERVER_STARTUP + DATA_STORE_STARTUP + FIRST_MSG_PROCESSED)
        .then(|()| check_client_list_data(data_store_addr, &data_send));

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
async fn read_data() {
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
    let sensor_values = [0.5];
    // let sensor_values = [0.0, 0.1, 0.2, 0.3];
    let run_data_server = data_server(
        ([127, 0, 0, 1], sub_port.port()),
        ([127, 0, 0, 1], data_port.port()),
    );
    let run_data_store = sleep(DATA_SERVER_STARTUP).then(|()| {
        data_store::server::run(data_server_addr, data_store_addr.port(), test_dir.path())
    });
    let send_sensor_value = sleep(DATA_SERVER_STARTUP + DATA_STORE_STARTUP)
        .then(|()| send_sensor_values(data_port.port(), &sensor_values, &data_send));
    let run_test = sleep(DATA_SERVER_STARTUP + DATA_STORE_STARTUP + FIRST_MSG_PROCESSED)
        .then(|()| check_client_get_data(data_store_addr, &sensor_values, &data_send));

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
