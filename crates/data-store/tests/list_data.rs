use std::iter;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use protocol::SensorMessage;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::time;

async fn fake_data_server(addr: SocketAddr) {
    let listener = TcpListener::bind(addr).await.unwrap();

    let (mut socket, _) = listener.accept().await.unwrap();

    let start = Instant::now();
    let pattern = iter::from_fn(move || {
        let x = start.elapsed().as_secs() % 180;
        let y = x as f32 * 0.5;
        Some(y)
    });
    let mut readings = pattern.map(|y| {
        protocol::Reading::LargeBedroom(protocol::large_bedroom::Reading::Bed(
            protocol::large_bedroom::bed::Reading::Temperature(y),
        ))
    });

    loop {
        time::sleep(Duration::from_secs(1)).await;
        let new = readings.next().unwrap();
        let mut msg = SensorMessage::<5>::new();
        msg.values.push(new);
        let bytes = msg.encode();
        socket.write_all(&bytes).await.unwrap();
    }
}

async fn test(data_store_addr: SocketAddr) {
    let mut client = data_store::Client::connect(data_store_addr).await.unwrap();
    let list = client.list_data().await.unwrap();

    assert!(list.iter().any(|r| {
        if let protocol::Reading::LargeBedroom(protocol::large_bedroom::Reading::Bed(
            protocol::large_bedroom::bed::Reading::Temperature(_),
        )) = r
        {
            true
        } else {
            false
        }
    }))
}

#[tokio::test]
async fn main() {
    let data_server_addr = SocketAddr::from(([127, 0, 0, 1], 3384));
    let data_store_addr = SocketAddr::from(([127, 0, 0, 1], 1294));

    let run_data_server = fake_data_server(data_server_addr);
    let run_data_store = data_store::run(data_server_addr, data_store_addr.port());
    let run_test = test(data_store_addr);

    use futures_concurrency::*;
    let res = (run_test, run_data_store, run_data_server).race().await;
}
