use defmt::{unwrap, warn};
use embassy_futures::select::{self, select};
use embassy_time::{with_timeout, Duration, Instant};
use protocol::small_bedroom::{self, bed};
use protocol::{affector, Affector, ErrorReport, SensorMessage};

use crate::channel::{PriorityValue, QueueItem, Queues};
use crate::sensors::slow;
use crate::usb_wrapper::{self, UsbHandle, UsbReceiver, UsbSender};

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



pub async fn handle(usb: UsbHandle<'_>, publish: &Queues, driver_orderers: &slow::DriverOrderers) {
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
    }
}

fn affector_list() -> affector::ListMessage<5> {
    let mut list = affector::ListMessage::<5>::empty();
    unwrap!(list.values.push(Affector::SmallBedroom(
        protocol::small_bedroom::Affector::Bed(bed::Affector::Sps30FanClean),
    )));
    unwrap!(list.values.push(Affector::SmallBedroom(
        protocol::small_bedroom::Affector::Bed(bed::Affector::MhzZeroPointCalib),
    )));
    unwrap!(list.values.push(Affector::SmallBedroom(
        protocol::small_bedroom::Affector::Bed(bed::Affector::Nau7802Calib),
    )));

    list
}

async fn send_messages(usb: UsbSender<'_>, publish: &Queues) {
    let mut buf = usb_wrapper::SendItem::new();
    unwrap!(buf.resize_default(buf.capacity()));
    let encoded_list_len = affector_list().encode_slice(&mut buf[1..]).len();
    buf[0] = protocol::Msg::<5>::AFFECTOR_LIST;
    buf.truncate(encoded_list_len + 1);
    usb.send(buf).await;

    loop {
        let mut buf = usb_wrapper::SendItem::new();
        unwrap!(buf.resize_default(buf.capacity()));
        let encoded_len = get_messages(publish, &mut buf).await.len();
        buf.truncate(encoded_len);
        usb.send(buf).await;
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
        }
    }
}
