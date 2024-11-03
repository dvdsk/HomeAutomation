use defmt::{unwrap, warn};
use embassy_futures::select::{self, select};
use embassy_time::{with_timeout, Duration, Instant};
use heapless::HistoryBuffer;
use protocol::small_bedroom::{self, bed};
use protocol::{affector, Affector, ErrorReport, SensorMessage};

use crate::channel::{PriorityValue, QueueItem, Queues};
use crate::sensors::slow;
use crate::usb_wrapper::{UsbHandle, UsbReceiver, UsbSender};

pub(crate) type SensMsg = SensorMessage<10>;
async fn collect_pending(publish: &Queues, reading: PriorityValue) -> SensorMessage<10> {
    let mut msg = SensMsg::default();
    let low_priority = reading.low_priority();
    unwrap!(msg.values.push(reading.value));

    if low_priority {
        let deadline = Instant::now() + Duration::from_millis(200);
        while msg.space_left() {
            let until = deadline.saturating_duration_since(Instant::now());
            match with_timeout(until, publish.receive_reading()).await {
                Ok(new) if new.low_priority() => {
                    unwrap!(msg.values.push(new.value));
                }
                Ok(new) => {
                    unwrap!(msg.values.push(new.value));
                    break;
                }
                Err(_timeout) => break,
            }
        }
    } else {
        while msg.space_left() {
            let Some(next) = publish.next_ready() else {
                break;
            };
            unwrap!(msg.values.push(next.value));
        }
    }
    msg
}

async fn get_messages<'a>(publish: &Queues, buf: &'a mut [u8]) -> &'a [u8] {
    let next = publish.receive().await;
    match next {
        QueueItem::Reading(reading) => {
            let msg = collect_pending(publish, reading).await;
            let encoded_len = msg.encode_slice(&mut buf[1..]).len();
            buf[0] = protocol::Msg::<0>::READINGS;
            &buf[..=encoded_len]
        }
        QueueItem::Error(error) => {
            let error = protocol::small_bedroom::Error::Bed(error.into());
            let error = protocol::Error::SmallBedroom(error);
            let encoded_len = ErrorReport::new(error).encode_slice(&mut buf[1..]).len();
            buf[0] = protocol::Msg::<0>::ERROR_REPORT;
            &buf[..=encoded_len]
        }
    }
}

pub async fn handle(
    usb: UsbHandle<'_>,
    publish: &Queues,
    driver_orderers: &slow::DriverOrderers,
) -> ! {
    const MIN_BETWEEN_COMM_ERRS: Duration = Duration::from_secs(30);
    let mut last_errors_at: HistoryBuffer<_, 5> = HistoryBuffer::new();

    loop {
        publish.clear().await;

        let (writer, reader) = usb.split();
        match select(
            send_messages(writer, publish),
            receive_orders(reader, &driver_orderers),
        )
        .await
        {
            select::Either::First(e) => warn!("Error while sending messages: {}", e),
            select::Either::Second(e) => warn!("Error receiving orders: {}", e),
        };

        let mut error_after = Instant::now();
        if last_errors_at
            .oldest_ordered()
            .rev()
            .copied()
            .all(|error: Instant| {
                let is_recent = error_after.duration_since(error) < MIN_BETWEEN_COMM_ERRS;
                error_after = error;
                is_recent
            })
        {
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

fn affector_list() -> affector::ListMessage<5> {
    let mut list = affector::ListMessage::<5>::empty();
    unwrap!(
        list.values.push(Affector::SmallBedroom(
            protocol::small_bedroom::Affector::Bed(bed::Affector::Sps30FanClean),
        )),
        "list is long enough"
    );
    unwrap!(
        list.values.push(Affector::SmallBedroom(
            protocol::small_bedroom::Affector::Bed(bed::Affector::MhzZeroPointCalib),
        )),
        "list is long enough"
    );
    unwrap!(
        list.values.push(Affector::SmallBedroom(
            protocol::small_bedroom::Affector::Bed(bed::Affector::Nau7802Calib),
        )),
        "list is long enough"
    );
    unwrap!(
        list.values.push(Affector::SmallBedroom(
            protocol::small_bedroom::Affector::Bed(bed::Affector::ResetNode),
        )),
        "list is long enough"
    );

    list
}

async fn send_messages(usb: UsbSender<'_>, publish: &Queues) {
    let mut buf = [0; protocol::Msg::<5>::max_size()];
    let encoded_len = affector_list().encode_slice(&mut buf[1..]).len();
    buf[0] = protocol::Msg::<5>::AFFECTOR_LIST;
    let to_send = &buf[..=encoded_len];
    usb.send(to_send).await;

    loop {
        let to_send = get_messages(publish, &mut buf).await;
        usb.send(to_send).await;
    }
}

async fn receive_orders(usb: UsbReceiver<'_>, driver_orderers: &slow::DriverOrderers) {
    defmt::debug!("ready to receive orders");
    loop {
        let mut read = usb.recv().await;
        let item = match affector::Affector::decode(&mut read) {
            Ok(item) => item,
            Err(e) => {
                defmt::error!("Could not decode affector: {}", e);
                continue;
            }
        };
        let Affector::SmallBedroom(small_bedroom::Affector::Bed(affector)) = item else {
            defmt::error!("Got affector for other node");
            continue;
        };

        defmt::info!("got affector order: {:?}", affector);
        match affector {
            bed::Affector::Nau7802Calib => {
                defmt::warn!("unimplemented affector: {:?}", affector)
            }
            bed::Affector::MhzZeroPointCalib => {
                driver_orderers.mhz.send(()).await;
            }
            bed::Affector::Sps30FanClean => {
                driver_orderers.sps.send(()).await;
            }
            bed::Affector::ResetNode => {
                defmt::info!("resetting node as orderd via affector");
                cortex_m::peripheral::SCB::sys_reset();
            }
        }
    }
}
