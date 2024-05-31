mod rooms;

use crate::system::System;
pub use protocol::Reading;
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcOffset};
use tokio::sync::broadcast;
use tokio::task::JoinSet;

// now_local works some of the time only... this replaces it with.......
// horrible hard coded time stuff. Chrono does provide reliable now_local
// however it has disadvantages (unsound + other flaws)
pub fn local_now() -> OffsetDateTime {
    let utc = OffsetDateTime::now_utc();
    let offset = tz::TimeZone::local()
        .unwrap()
        .find_current_local_time_type()
        .unwrap()
        .ut_offset();

    let offset = UtcOffset::from_whole_seconds(offset).unwrap();
    utc.to_offset(offset)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Sensor(Reading),
    WakeUp,
}

struct RestrictedSystem {
    allowed_lights: Vec<&'static str>,
    system: System,
}

impl RestrictedSystem {
    async fn all_lamps_ct(&mut self, ct: u16, bri: u8) {
        for name in &self.allowed_lights {
            self.system.lights.set_ct(name, bri, ct).await.unwrap();
        }
    }

    async fn all_lamps_off(&mut self) {
        for name in &self.allowed_lights {
            self.system.lights.single_off(name).await.unwrap();
        }
    }

    async fn all_lamps_on(&mut self) {
        for name in &self.allowed_lights {
            self.system.lights.single_on(name).await.unwrap();
        }
    }
}

pub fn start(
    subscribed: [broadcast::Receiver<Event>; 3],
    sender: broadcast::Sender<Event>,
    system: System,
) -> JoinSet<()> {
    tracing::info!("starting");
    let mut tasks = JoinSet::new();
    let [rx1, rx2, rx3] = subscribed;

    let restricted = RestrictedSystem {
        system: system.clone(),
        allowed_lights: vec![
            "large_bedroom:cabinet",
            "large_bedroom:ceiling",
            "large_bedroom:desk",
            "large_bedroom:wardrobe",
            "large_bedroom:bed",
        ],
    };
    tasks.spawn(rooms::large_bedroom::run(rx1, sender.clone(), restricted));

    let restricted = RestrictedSystem {
        system: system.clone(),
        allowed_lights: vec![
            "small_bedroom:ceiling",
            "small_bedroom:bureau",
            "small_bedroom:piano",
        ],
    };
    tasks.spawn(rooms::small_bedroom::run(rx2, sender.clone(), restricted));

    let restricted = RestrictedSystem {
        system: system.clone(),
        allowed_lights: vec!["kitchen:ceiling", "kitchen:hood_left", "kitchen:hood_right"],
    };
    tasks.spawn(rooms::kitchen::run(rx3, sender, restricted));

    tasks
}
