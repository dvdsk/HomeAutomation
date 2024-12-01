use jiff::{Span, Zoned};
use std::{sync::mpsc, thread, time::Duration};
use tokio::sync::broadcast;
use tracing::{error, info};

use byteorder::{BigEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use sled;

use crate::{controller::Event, time::to_datetime};

pub mod wakeup;
pub use wakeup::WakeUp;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could store/edit job on disk")]
    DbError(#[from] sled::Error),
    #[error("Could not inform waker about new job")]
    CommError(#[from] mpsc::SendError<()>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Job {
    pub time: Zoned,
    pub event: Event,
    /// how long after the time was missed the job
    /// should still go off
    pub expiration: Option<Duration>,
}

impl Job {
    fn at_next(
        hour: i8,
        min: i8,
        event: Event,
        expiration: Option<Duration>,
    ) -> Job {
        Job {
            time: to_datetime(hour, min),
            event,
            expiration,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Jobs {
    waker_tx: mpsc::Sender<()>,
    list: JobList,
}

#[derive(Debug, Clone)]
struct JobList {
    db: sled::Tree,
}

use mpsc::RecvTimeoutError::*;
fn waker(
    mut job_list: JobList,
    event_tx: broadcast::Sender<Event>,
    waker_rx: mpsc::Receiver<()>,
) {
    loop {
        // This can fail
        // TODO make sure an non waking error alarm is send to the user
        if let Some((id, current_job)) = job_list.peek_next() {
            let now = crate::time::now();
            let timeout = &current_job.time - &now;
            if let Some(expiration) = current_job.expiration {
                if now > &current_job.time + Span::try_from(expiration).unwrap()
                {
                    error!("skipping job too far in the past");
                    job_list.remove_job(id).unwrap();
                    continue; // job too far in the past, skip and get next
                }
            }
            let timeout =
                Duration::try_from(timeout).unwrap_or(Duration::from_secs(0));
            info!("next job is in: {} seconds", timeout.as_secs());

            // do we send out the event or should we add or remove a job?
            match waker_rx.recv_timeout(timeout) {
                Ok(_) => continue, // new job entered, restart loop
                Err(Disconnected) => return,
                Err(Timeout) => {
                    // time to send the job event
                    event_tx
                        .send(current_job.event)
                        .expect("controller should listen on this");
                    job_list.remove_job(id).unwrap();
                    continue; //get next job
                }
            }
        } else {
            //no alarm to wait on, wait for instructions
            info!("no job in the future");
            //A message through the mpsc signals an alarm has been added
            match waker_rx.recv() {
                // jobs were added or removed, go back and start waiting on them
                Ok(_) => break,
                // can't have timed out thus program should exit
                Err(_) => return,
            }
        }
    }
}

impl Jobs {
    pub fn setup(
        event_tx: broadcast::Sender<Event>,
        db: sled::Db,
    ) -> Result<(Self, thread::JoinHandle<()>), Error> {
        let job_list = JobList {
            db: db.open_tree("jobs")?,
        };

        let (waker_tx, waker_rx) = mpsc::channel();
        let waker_db_copy = job_list.clone();
        let waker_thread =
            thread::spawn(move || waker(waker_db_copy, event_tx, waker_rx));

        Ok((Self { list: job_list, waker_tx }, waker_thread))
    }

    // we decrease the time till the job until there is a place in the database
    // as only one job can fire at the time, after a job gets a timeslot
    // it is never changed
    pub async fn add_job(&self, to_add: Job) -> Result<i64, Error> {
        let id = self.list.add_job(to_add).await?;
        //signal waker to update its next alarm
        // TODO this feels like it shouldn't be here in this way
        self.waker_tx.send(())?;
        Ok(id)
    }
    pub fn remove_job(&self, to_remove: i64) -> Result<Option<Job>, Error> {
        let removed_alarm = self.list.remove_job(to_remove)?;
        self.waker_tx.send(())?; //signal waker to update its next alarm
        Ok(removed_alarm)
    }
    pub fn get(&self, id: i64) -> Result<Option<Job>, Error> {
        self.list.get_alarm(id)
    }

    // pub fn list(&self) -> Vec<(u64, Job)> {
    //     let start: &[u8] = &[0];
    //     let alarms = self.list.db.range(start..);
    //
    //     let mut list = Vec::new();
    //     for (key, alarm) in alarms.filter_map(Result::ok) {
    //         let alarm = bincode::deserialize(&alarm).unwrap();
    //         let key = key.as_ref().read_u64::<BigEndian>().unwrap();
    //         list.push((key, alarm));
    //     }
    //     list
    // }
}

impl JobList {
    pub fn get_alarm(&self, id: i64) -> Result<Option<Job>, Error> {
        Ok(self
            .db
            .get(id.to_be_bytes())?
            .map(|k| bincode::deserialize::<Job>(&k).unwrap()))
    }

    // we decrease the time till the alarm until there is a place in the database
    // as only one alarm can fire at the time, after an alarm gets a timeslot
    // it is never changed
    /// return the key for the alarm
    async fn add_job(&self, to_add: Job) -> Result<i64, Error> {
        let mut timestamp = to_add.time.timestamp().as_millisecond();
        let job = bincode::serialize(&to_add).unwrap();

        // create alarm entry if there is no alarm at this timestamp yet
        // if there is already an alarm scheduled, change the key for this one
        // until there is a spot free
        // TODO check how this works and expand sled documentation for fail cases
        while let Err(_old_event) = self.db.compare_and_swap(
            timestamp.to_be_bytes(),
            None as Option<&[u8]>,
            Some(job.clone()),
        )? {
            // create unique key
            timestamp -= 1;
        }
        self.db.flush_async().await?;
        Ok(timestamp)
    }

    pub fn remove_job(&self, to_remove: i64) -> Result<Option<Job>, Error> {
        let old_job = self
            .db
            .remove(to_remove.to_be_bytes())?
            .map(|k| bincode::deserialize::<Job>(&k).unwrap());
        self.db.flush()?;
        Ok(old_job)
    }

    /// calculate time to the earliest job, remove it from the list if the
    /// current time is later than the job
    fn peek_next(&mut self) -> Option<(i64, Job)> {
        // get earliest job time in db
        match self.db.get_gt(0u64.to_be_bytes()) {
            Ok(entry) => {
                if let Some((id, job)) = entry {
                    let id = id.as_ref().read_i64::<BigEndian>().unwrap();
                    let job = bincode::deserialize(&job).unwrap();
                    Some((id, job))
                } else {
                    None
                }
            }
            Err(error) => {
                error!("Could not retrieve next alarm: {:?}", error);
                None
            }
        }
    }
}
