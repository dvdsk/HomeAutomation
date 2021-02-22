use super::{Jobs, Job, Action};
// use crate::errors::Error;
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
    pub async fn set_tomorrow(&self, hour: u8, min: u8) -> Result<(), Error> {
        self.0.lock().unwrap().set_tomorrow(hour, min).await
    }
    pub async fn set_usually(&self, hour: u8, min: u8) -> Result<(), Error> {
        self.0.lock().unwrap().set_usually(hour, min).await
    }
}

struct Inner {
    db: sled::Tree,
    pub tomorrow: Option<(u8, u8)>,
    pub usually: Option<(u8, u8)>,
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

    fn register_new_job(&mut self, id: u64) -> Result<(), Error> {
        self.job_id = Some(id);
        let bytes = bincode::serialize(&id).unwrap();
        if let Some(bytes) = self.db.insert("job_id", bytes)? {
            let old_job_id = bincode::deserialize(&bytes).unwrap();
            self.jobs.remove_alarm(old_job_id)?;
        }
        Ok(())
    }

    async fn add_new_job(&mut self, job: Job) -> Result<(),Error> {
        let id = self.jobs
            .add_alarm(job)
            .await?;
        self.register_new_job(id)?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(),Error> {
        if let Some((hour,min)) = self.usually {
            let job = wakeup_job(hour, min);
            let add = self.add_new_job(job);
            smol::block_on(add)?;
        }
        self.tomorrow = None;
        Ok(())
    }

    pub async fn set_tomorrow(&mut self, hour: u8, min: u8) -> Result<(),Error> {
        let job = wakeup_job(hour, min);
        self.tomorrow = Some((hour,min));
        self.add_new_job(job).await?;
        Ok(())
    }

    pub async fn set_usually(&mut self, hour: u8, min: u8) -> Result<(), Error> {
        let job = wakeup_job(hour, min);
        self.usually = Some((hour,min));
        self.add_new_job(job).await?;
        Ok(())
    }
}

fn wakeup_job(hour: u8, min: u8) -> Job {
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
