use core::fmt;

use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_sync::priority_channel::{self, PriorityChannel};
use embassy_time::Instant;
use heapless::Vec;
use protocol::large_bedroom::bed::Reading;

use crate::error_cache;

struct ErrorEvent {
    error: error_cache::Error,
    at: Instant,
}

pub struct Queues {
    sensor_queue: PriorityChannel<NoopRawMutex, PriorityValue, priority_channel::Max, 20>,
    error_queue: Channel<NoopRawMutex, error_cache::Error, 20>,
    recent_errors: Mutex<NoopRawMutex, Vec<error_cache::Error, 5>>,
}

impl Queues {
    pub fn new() -> Self {
        Self {
            sensor_queue: PriorityChannel::new(),
            error_queue: Channel::new(),
            recent_errors: Mutex::new(Vec::new()),
        }
    }

    pub async fn clear(&self) {
        while let Ok(_) = self.sensor_queue.try_receive() {}
        self.recent_errors.lock().await.clear();
    }

    pub async fn receive(&self) -> PriorityValue {
        self.sensor_queue.receive().await
    }

    pub fn next_ready(&self) -> Option<PriorityValue> {
        self.sensor_queue.try_receive().ok()
    }

    pub fn queue_error(&self, _error: error_cache::Error) {
        // let mut recent_errors = unwrap!(self.recent_errors.try_lock());
        //
        // let mut to_remove: Vec<usize, 20> = Vec::new();
        // for (idx, event) in recent_errors.iter().enumerate() {
        //     if event.at.elapsed() > Duration::from_secs(60) {
        //         unwrap!(to_remove.push(idx));
        //     } else if event.error == error {
        //         return;
        //     }
        // }
        //
        // for idx in to_remove.iter().rev() {
        //     recent_errors.swap_remove(*idx);
        // }
        // let full = self.error_queue.try_send(error).is_err();

        // if !full {
        //     let _ignore_full = recent_errors.push(ErrorEvent {
        //         error,
        //         at: Instant::now(),
        //     });
        // }
    }

    pub fn send_p0(&self, value: Reading) {
        let entry = PriorityValue {
            priority: 0,
            value: protocol::Reading::LargeBedroom(protocol::large_bedroom::Reading::Bed(value)),
        };
        let _ignore_full = self.sensor_queue.try_send(entry);
    }
    pub fn send_p1(&self, value: Reading) {
        let entry = PriorityValue {
            priority: 1,
            value: protocol::Reading::LargeBedroom(protocol::large_bedroom::Reading::Bed(value)),
        };
        let _ignore_full = self.sensor_queue.try_send(entry);
    }

    pub fn send_p2(&self, value: Reading) {
        let entry = PriorityValue {
            priority: 2,
            value: protocol::Reading::LargeBedroom(protocol::large_bedroom::Reading::Bed(value)),
        };
        let _ignore_full = self.sensor_queue.try_send(entry);
    }
}

/// Higher prio will be send earlier
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
