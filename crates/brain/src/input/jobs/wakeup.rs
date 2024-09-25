use crate::controller::Event;

use super::{Job, Jobs};
use chrono::{DateTime, Local, Timelike, Utc};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::{broadcast, Mutex};
use tokio::task;
use tracing::{error, info};

// TODO FIXME multiple things left to do:
// - usually and tomorrow are not written to db
// - setting usually none goes wrong if tomorrow is not none
// - recheck set tomorrow and set usually

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error modifying wakeup alarms: {0}")]
    DbError(#[from] sled::Error),
    #[error("Could not create/edit wakeup job")]
    JobError(#[from] super::Error),
}

async fn reset_on_wakeup(wake_up: WakeUp, mut event_rx: broadcast::Receiver<Event>) {
    loop {
        match event_rx.recv().await {
            Ok(Event::WakeUp) => {
                if let Err(e) = wake_up.reset() {
                    /* TODO: this should make more "noise" then this <28-04-24, dvdsk> */
                    error!("Could not reset wakeup system for next wake: {e}");
                }
            }
            Err(broadcast::error::RecvError::Closed) => return,
            _ => (),
        }
    }
}

#[derive(Clone)]
pub struct WakeUp(Arc<Mutex<Inner>>);
impl WakeUp {
    /// # Important
    /// The receiver must be subscribed before the jobs system is started
    /// or it can miss alarms
    pub fn setup(
        db: sled::Db,
        jobs: Jobs,
        event_rx: broadcast::Receiver<Event>,
    ) -> Result<Self, Error> {
        let inner = Inner::setup(db, jobs)?;
        let inner = Arc::new(Mutex::new(inner));
        let this = Self(inner.clone());
        task::spawn(reset_on_wakeup(this, event_rx));

        Ok(Self(inner))
    }
    pub async fn tomorrow(&self) -> Option<(u8, u8)> {
        self.0.lock().await.tomorrow
    }
    pub async fn usually(&self) -> Option<(u8, u8)> {
        self.0.lock().await.usually
    }
    pub fn reset(&self) -> Result<(), Error> {
        self.0.blocking_lock().reset()?;
        info!("alarm system reset correctly");
        Ok(())
    }
    pub async fn set_tomorrow(&self, time: Time) -> Result<(), Error> {
        self.0.lock().await.set_tomorrow(time).await?;
        info!("alarm tomorrow set to: time");
        Ok(())
    }
    pub async fn set_usually(&self, time: Time) -> Result<(), Error> {
        self.0.lock().await.set_usually(time).await?;
        info!("alarm usually set to: time");
        Ok(())
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
            // we want bincode to deserialize to Option<(u8,u8)> not (u8,u8)
            .and_then(|b| bincode::deserialize(&b).unwrap());
        let job_id = db.get("job_id")?.map(|b| bincode::deserialize(&b).unwrap());
        let next_alarm = job_id
            .as_ref()
            .and_then(|id| jobs.get(*id).unwrap())
            .map(|job| job.time.into())
            .map(|t: DateTime<Local>| t)
            .map(|t| (t.hour() as u8, t.minute() as u8));
        let tomorrow = next_alarm.filter(|tomorrow| Some(*tomorrow) != usually);

        Ok(Self {
            db,
            tomorrow,
            usually,
            job_id,
            jobs,
        })
    }

    /// stores a new job
    fn save_job_id(&mut self, id: u64) -> Result<(), Error> {
        self.job_id = Some(id);
        let bytes = bincode::serialize(&id).unwrap();
        if let Some(bytes) = self.db.insert("job_id", bytes)? {
            let old_job_id = bincode::deserialize(&bytes).unwrap();
            self.jobs.remove_alarm(old_job_id)?;
        }
        self.db.flush().unwrap();
        Ok(())
    }

    fn save_usually(&mut self, usually: &Time) -> Result<(), Error> {
        let bytes = bincode::serialize(usually).unwrap();
        self.db.insert("usually", bytes)?;
        self.db.flush()?;
        Ok(())
    }

    async fn replace_job(&mut self, job: Job) -> Result<(), Error> {
        let id = self.jobs.add_alarm(job).await?;
        self.save_job_id(id)?;
        Ok(())
    }

    fn remove_job(&mut self) -> Result<(), Error> {
        self.jobs.remove_alarm(self.job_id.unwrap())?;
        self.job_id = None;
        Ok(())
    }

    /// reset the alarm, if there is a usual alarm
    /// time we set that, otherwise remove all
    pub fn reset(&mut self) -> Result<(), Error> {
        if let Some((hour, min)) = self.usually {
            let job = job_from(hour, min);
            let add = self.replace_job(job);
            let rt = Runtime::new().unwrap();
            rt.block_on(add)?;
        }
        self.tomorrow = None;
        Ok(())
    }

    pub async fn set_tomorrow(&mut self, time: Time) -> Result<(), Error> {
        match time {
            None => self.reset()?,
            Some((hour, min)) => {
                let job = job_from(hour, min);
                self.replace_job(job).await?;
            }
        }
        self.tomorrow = time;
        Ok(())
    }

    pub async fn set_usually(&mut self, time: Time) -> Result<(), Error> {
        self.save_usually(&time)?;
        match time {
            Some((hour, min)) => {
                let job = job_from(hour, min);
                self.replace_job(job).await?;
            }
            None => {
                if self.job_id.is_some() && self.tomorrow.is_none() {
                    self.remove_job()?;
                }
            }
        }
        self.usually = time;
        Ok(())
    }
}

fn job_from(hour: u8, min: u8) -> Job {
    Job {
        time: to_datetime(hour, min),
        event: Event::WakeUp,
        expiration: Some(Duration::from_secs(3 * 60 * 60)),
    }
}

fn to_datetime(hour: u8, min: u8) -> DateTime<Utc> {
    let (hour, min) = (hour as u32, min as u32);
    let now = Local::now();
    let today = now.date_naive();
    let alarm = today.and_hms_opt(hour, min, 0).unwrap();

    if alarm < now.naive_local() {
        let tomorrow = now.date_naive().succ_opt().unwrap();
        tomorrow
            .and_hms_opt(hour, min, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .with_timezone(&Utc)
    } else {
        alarm.and_local_timezone(Local).unwrap().with_timezone(&Utc)
    }
}
