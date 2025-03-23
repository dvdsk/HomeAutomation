use embassy_futures::select::{self, Either};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_sync::priority_channel::{self, PriorityChannel};
use embassy_time::{Duration, Instant};
use heapless::HistoryBuffer;
use protocol::large_bedroom::bed::Reading;

pub struct Queues {
    sensor_queue: PriorityChannel<
        ThreadModeRawMutex,
        PriorityValue,
        priority_channel::Max,
        20,
    >,
    error_queue: Channel<ThreadModeRawMutex, sensors::Error, 20>,
    recent_errors: Mutex<ThreadModeRawMutex, HistoryBuffer<sensors::Error, 20>>,
    recent_is_since: Instant,
}

pub enum QueueItem {
    Reading(PriorityValue),
    Error(sensors::Error),
}

impl Queues {
    pub const fn new() -> Self {
        Self {
            sensor_queue: PriorityChannel::new(),
            error_queue: Channel::new(),
            recent_errors: Mutex::new(HistoryBuffer::new()),
            recent_is_since: Instant::MIN, // time since CPU start
        }
    }

    pub async fn clear(&self) {
        while self.sensor_queue.try_receive().is_ok() {}
        self.recent_errors.lock().await.clear();
    }

    pub async fn receive(&self) -> QueueItem {
        if let Ok(val) = self.sensor_queue.try_receive() {
            return QueueItem::Reading(val);
        }

        let race = select::select(
            self.sensor_queue.receive(),
            self.error_queue.receive(),
        );
        match race.await {
            Either::First(reading) => QueueItem::Reading(reading),
            Either::Second(error) => QueueItem::Error(error),
        }
    }

    pub async fn receive_reading(&self) -> PriorityValue {
        self.sensor_queue.receive().await
    }

    pub fn next_ready(&self) -> Option<PriorityValue> {
        self.sensor_queue.try_receive().ok()
    }

    pub fn queue_error(&self, error: sensors::Error) {
        // unwrap safe: is not used in interrupts
        let mut recent_errors = defmt::unwrap!(self.recent_errors.try_lock());
        if self.recent_is_since.elapsed() > Duration::from_secs(60 * 5) {
            recent_errors.clear();
        }
        if recent_errors.contains(&error) {
            return;
        }

        defmt::debug!("queueing error: {:?}", error);
        let Ok(()) = self.error_queue.try_send(error.clone()) else {
            return;
        };
        recent_errors.write(error);
    }

    pub fn send_p0(&self, value: Reading) {
        use protocol::large_bedroom::Reading::Bed;
        let entry = PriorityValue {
            priority: 0,
            value: protocol::Reading::LargeBedroom(Bed(value)),
        };
        let _ignore_full = self.sensor_queue.try_send(entry);
    }
    pub fn send_p1(&self, value: Reading) {
        use protocol::large_bedroom::Reading::Bed;
        let entry = PriorityValue {
            priority: 1,
            value: protocol::Reading::LargeBedroom(Bed(value)),
        };
        let _ignore_full = self.sensor_queue.try_send(entry);
    }

    pub fn send_p2(&self, value: Reading) {
        use protocol::large_bedroom::Reading::Bed;
        let entry = PriorityValue {
            priority: 2,
            value: protocol::Reading::LargeBedroom(Bed(value)),
        };
        let _ignore_full = self.sensor_queue.try_send(entry);
    }
}

/// Higher priority will be send earlier
pub struct PriorityValue {
    priority: u8,
    pub value: protocol::Reading,
}

impl PriorityValue {
    pub fn low_priority(&self) -> bool {
        self.priority < 2
    }
}

impl Eq for PriorityValue {}
impl PartialEq for PriorityValue {
    fn eq(&self, other: &Self) -> bool {
        self.priority.eq(&other.priority)
    }
}

impl PartialOrd for PriorityValue {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.priority.cmp(&other.priority))
    }
}

impl Ord for PriorityValue {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}
