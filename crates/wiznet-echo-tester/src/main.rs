use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:6969").unwrap();

    println!("waiting for incoming connections");
    for conn in listener.incoming() {
        let mut conn = match conn {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("error with incoming connection: {e}");
                continue;
            },
        };

        println!("Got connection from: {:?}", conn.peer_addr());
        let mut buf = [0u8; 100];
        for i in 0u64.. {
            buf[0..8].copy_from_slice(&i.to_be_bytes());
            if let Err(e) = conn.write_all(&buf) {
                eprintln!("Could not write to incoming conn: {e}");
                break;
            }
            if let Err(e) = conn.read_exact(&mut buf) {
                eprintln!("Could not read echo from incoming conn: {e}");
                break;
            }

            assert_eq!(i, u64::from_be_bytes(buf[0..8].try_into().unwrap()));

            if i % 1000 == 0 {
                println!("Performed {i} echos");
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}
