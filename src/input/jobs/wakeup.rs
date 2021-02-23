use super::{Jobs, Job, Action};
use crate::controller::Event;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use chrono::{Utc, Local, DateTime, Timelike};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error modifying wakeup alarms: {0}")]
    DbError(#[from] sled::Error),
    #[error("Could not create/edit wakeup job")]
    JobError(#[from] super::Error),
}

#[derive(Clone)]
pub struct WakeUp(Arc<Mutex<Inner>>); 
impl WakeUp {
    pub fn setup(db: sled::Db, jobs: Jobs) -> Result<Self, Error> {
        let inner = Inner::setup(db, jobs)?;
        let inner = Arc::new(Mutex::new(inner));
        Ok(Self(inner))
    }
    pub fn tomorrow(&self) -> Option<(u8,u8)> {
        self.0.lock().unwrap().tomorrow
    }
    pub fn usually(&self) -> Option<(u8,u8)> {
        self.0.lock().unwrap().usually
    }
    pub fn reset(&self) -> Result<(), Error> {
        self.0.lock().unwrap().reset()
    }
    pub async fn set_tomorrow(&self, time: Time) -> Result<(), Error> {
        self.0.lock().unwrap().set_tomorrow(time).await
    }
    pub async fn set_usually(&self, time: Time) -> Result<(), Error> {
        self.0.lock().unwrap().set_usually(time).await
    }
}

type Time = Option<(u8, u8)>;
struct Inner {
    db: sled::Tree,
    pub tomorrow: Time,
    pub usually: Time,
    job_id: Option<u64>,
    jobs: Jobs,
}

impl Inner {
    pub fn setup(db: sled::Db, jobs: Jobs) -> Result<Self, Error> {
        let db = db.open_tree("wakeup")?;
        let usually = db
            .get("usually")?
            .map(|b| bincode::deserialize(&b).unwrap());
        let job_id = db
            .get("job_id")?
            .map(|b| bincode::deserialize(&b).unwrap());
        let tomorrow = job_id.as_ref()
            .map(|id| jobs.get(*id).ok())
            .flatten().flatten()
            .map(|job| job.time.into())
            .map(|t: DateTime<Local>| (t.hour() as u8, t.minute() as u8))
            .filter(|tomorrow| Some(*tomorrow) != usually);

        Ok(Self {
            db,
            tomorrow,
            usually,
            job_id,
            jobs,
        })
    }

    fn save_job_id(&mut self, id: u64) -> Result<(), Error> {
        self.job_id = Some(id);
        let bytes = bincode::serialize(&id).unwrap();
        if let Some(bytes) = self.db.insert("job_id", bytes)? {
            let old_job_id = bincode::deserialize(&bytes).unwrap();
            self.jobs.remove_alarm(old_job_id)?;
        }
        Ok(())
    }

    async fn replace_current(&mut self, job: Job) -> Result<(),Error> {
        let id = self.jobs
            .add_alarm(job)
            .await?;
        self.save_job_id(id)?;
        Ok(())
    }

    async fn remove_current(&mut self) -> Result<(), Error> {
        self.jobs.remove_alarm(self.job_id.unwrap())?;
        self.job_id = None;
        Ok(())
    }

    /// reset the alarm, if their is a usual alarm
    /// time we set that, otherwise remove all
    pub fn reset(&mut self) -> Result<(),Error> {
        if let Some((hour,min)) = self.usually {
            let job = job_from(hour, min);
            let add = self.replace_current(job);
            smol::block_on(add)?;
        } else {
            let remove = self.remove_current();
            smol::block_on(remove)?;
        }
        self.tomorrow = None;
        Ok(())
    }

    pub async fn set_tomorrow(&mut self, time: Time) -> Result<(),Error> {
        match time {
            None => self.reset()?,
            Some((hour, min)) => {
                let job = job_from(hour, min);
                self.replace_current(job).await?;
            }
        }
        self.tomorrow = time;
        Ok(())
    }

    pub async fn set_usually(&mut self, time: Time) -> Result<(), Error> {
        match time {
            None => self.reset()?,
            Some((hour, min)) => {
                let job = job_from(hour, min);
                self.replace_current(job).await?;
            }
        }
        self.usually = time;
        Ok(())
    }
}

fn job_from(hour: u8, min: u8) -> Job {
    Job {
        time: to_datetime(hour,min),
        action: Action::SendEvent(Event::WakeUp),
        expiration: Some(Duration::from_secs(3*60*60)),
    }
}

fn to_datetime(hour: u8, min: u8) -> DateTime<Utc> {
    let (hour, min) = (hour as u32, min as u32);
    let now = Local::now();
    let alarm = now
        .with_hour(hour).unwrap()
        .with_minute(min).unwrap();

    if alarm < now {
        let tomorrow = now.date().succ();
        tomorrow.and_hms(hour, min, 0).into()
    } else {
        alarm.into()
    }
}
