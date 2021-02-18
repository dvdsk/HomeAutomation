use super::{Jobs, Job, Action};
use crate::errors::Error;
use crate::controller::Event;
use std::time::Duration;
use chrono::{Utc, Local, DateTime, Timelike};

#[derive(Clone)]
pub struct WakeUp {
    db: sled::Tree,
    tomorrow: Option<(u8, u8)>,
    usually: Option<(u8, u8)>,
    job_id: Option<u64>,
    jobs: Jobs,
}

impl WakeUp {
    pub fn setup(db: sled::Db, jobs: Jobs) -> Result<Self, Error> {
        let db = db.open_tree("wakeup")?;
        let tomorrow = db
            .get("tomorrow")
            .unwrap()
            .map(|b| bincode::deserialize(&b).unwrap());
        let usually = db
            .get("usually")
            .unwrap()
            .map(|b| bincode::deserialize(&b).unwrap());
        let job_id = db
            .get("job_id")
            .unwrap()
            .map(|b| bincode::deserialize(&b).unwrap());

        Ok(Self {
            db,
            tomorrow,
            usually,
            job_id,
            jobs,
        })
    }

    pub fn reset(&mut self) {
        if let Some(hour_min) = self.usually {
            let job = Job {
                time: to_datetime(hour_min),
                action: Action::SendEvent(Event::WakeUp),
                expiration: Some(Duration::from_secs(3*60*60)),
            };
            self.jobs.add_alarm(job);
        }
    }
}

fn to_datetime((hour, min): (u8, u8)) -> DateTime<Utc> {
    let (hour, min) = (hour as u32, min as u32);
    let now = Utc::now();

    let tomorrow = now.date().succ();
    tomorrow.and_hms(hour, min, 0)
}
