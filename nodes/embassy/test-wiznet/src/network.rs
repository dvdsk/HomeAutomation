use defmt::{debug, info, warn};
use embassy_executor::task;
use embassy_net::tcp::TcpSocket;
use embassy_net::{Ipv4Address, Stack};
use embassy_time::{Duration, Timer};

use embedded_io_async::Write;
use embedded_io_async::Read;

#[task]
pub async fn handle(
    stack: Stack<'static>,
) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(5)));
    socket.set_keep_alive(Some(Duration::from_secs(1)));
    let host_addr = Ipv4Address::new(192, 168, 1, 43);
    let host_port = 6969;

    let mut sequential_errors = 0;

    debug!("Configured socket and connecting");
    loop {
        debug!("socket state: {:?}", socket.state());
        if let Err(e) = socket.connect((host_addr, host_port)).await {
            if sequential_errors > 3 {
                defmt::info!("failing to reconnect, resetting node");
                defmt::flush();
                Timer::after_secs(1).await;
                cortex_m::peripheral::SCB::sys_reset();
            }
            warn!("connect error: {}", e);
            sequential_errors += 1;
            Timer::after_secs(5).await;
            continue;
        }

        info!("(re-)connected");
        sequential_errors = 0;
        // Prevent out-dated data from being send

        let mut buf = [0u8; 100];
        loop {
            let Ok(_) = socket.read_exact(&mut buf).await else {
                break
            };
            let Ok(_) = socket.write_all(&buf).await else {
                break
            };
        }

        // Or the socket will hang for a while waiting to close this makes sure
        // we can reconnect instantly
        socket.abort();
        Timer::after_secs(60).await; // Experiment: does this help?
    }
}
