use defmt::warn;
use embassy_time::{Duration, Instant};
use heapless::HistoryBuffer;
use protocol::{SensorMessage, affector};
use usb_bridge_client::{NoSpaceInQueue, UsbHandle, UsbSender};

pub async fn handle(usb: UsbHandle<'_>) -> ! {
    const MIN_BETWEEN_COMM_ERRS: Duration = Duration::from_secs(30);
    let mut last_errors_at: HistoryBuffer<_, 5> = HistoryBuffer::new();

    loop {
        crate::PUBLISH.clear();

        let (writer, _) = usb.split();
        send_messages(writer).await;
        warn!("Error while sending messages");

        let mut error_after = Instant::now();
        if last_errors_at.oldest_ordered().rev().copied().all(
            |error: Instant| {
                let is_recent =
                    error_after.duration_since(error) < MIN_BETWEEN_COMM_ERRS;
                error_after = error;
                is_recent
            },
        ) {
            defmt::error!(
                "Something is terribly wrong with the connection, \
                5 errors occured each within 30 seconds of the previous. \
                Resetting entire node"
            );
            cortex_m::peripheral::SCB::sys_reset();
        } else {
            last_errors_at.write(Instant::now());
        }
    }
}

const AFFECTOR_LIST_MAX_SIZE: usize = protocol::Msg::<5>::max_size();
pub fn affector_list() -> heapless::Vec<u8, AFFECTOR_LIST_MAX_SIZE> {
    let list = affector::ListMessage::<5>::empty();
    let mut buf = heapless::Vec::new();
    defmt::unwrap!(buf.resize_default(AFFECTOR_LIST_MAX_SIZE));
    let encoded_len = list.encode_slice(&mut buf).len();
    buf.truncate(encoded_len);
    buf
}

async fn collect_msg() -> SensorMessage<10> {
    let mut msg = SensorMessage::<10>::default();
    let reading = crate::PUBLISH.receive().await;
    defmt::unwrap!(msg.values.push(reading));
    msg
}

async fn send_messages(usb: UsbSender<'_>) {
    let mut buf = [0; protocol::Msg::<5>::max_size()];
    loop {
        let msg = collect_msg().await;
        let encoded_len = msg.encode_slice(&mut buf[1..]).len();
        buf[0] = protocol::Msg::<0>::READINGS;
        let to_send = &buf[..=encoded_len];
        while let Err(NoSpaceInQueue) = usb.send(to_send, true).await {
            usb.wait_till_queue_free().await;
        }
    }
}
