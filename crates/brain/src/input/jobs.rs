use std::{sync::mpsc, thread, time::Duration};

use byteorder::{BigEndian, ReadBytesExt};
use jiff::{Span, Zoned};
use mpsc::RecvTimeoutError::*;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tracing::{error, info};

use crate::{controller::Event, time::to_datetime};

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("Could not store/edit job on disk")]
    DbError(#[from] sled::Error),
    #[error("Could not inform event timer about new job")]
    CommError(#[from] mpsc::SendError<()>),
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Job {
    pub(crate) time: Zoned,
    pub(crate) event: Event,
    /// how long after the time was missed the job
    /// should still be executed
    pub(crate) expiration: Option<Duration>,
}

impl Job {
    pub(crate) fn at_next(
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
pub(crate) struct Jobs {
    job_change_tx: mpsc::Sender<()>,
    list: JobList,
}

#[derive(Debug, Clone)]
struct JobList {
    db: sled::Tree,
}

impl Jobs {
    pub(crate) fn setup(
        event_tx: broadcast::Sender<Event>,
        db: sled::Db,
    ) -> Result<Self, Error> {
        let job_list = JobList::new(db)?;

        let (job_change_tx, job_change_rx) = mpsc::channel();
        let job_list_clone = job_list.clone();
        thread::spawn(move || {
            event_timer(job_list_clone, event_tx, job_change_rx)
        });

        Ok(Self {
            list: job_list,
            job_change_tx,
        })
    }

    // we decrease the time till the job until there is a place in the database
    // as only one job can fire at the time, after a job gets a timeslot
    // it is never changed
    pub(crate) async fn add_job(&self, to_add: Job) -> Result<i64, Error> {
        let id = self.list.add_job(to_add).await?;
        //signal event timer to update its next job
        self.job_change_tx.send(())?;
        Ok(id)
    }
    pub(crate) fn remove_job(&self, to_remove: i64) -> Result<Option<Job>, Error> {
        let removed_job = self.list.remove_job(to_remove)?;
        //signal event timer to update its next job
        self.job_change_tx.send(())?;
        Ok(removed_job)
    }
    pub(crate) fn get(&self, id: i64) -> Result<Option<Job>, Error> {
        self.list.get_job(id)
    }
}

impl JobList {
    fn new(db: sled::Db) -> Result<Self, Error> {
        Ok(JobList {
            db: db.open_tree("jobs")?,
        })
    }

    fn get_job(&self, id: i64) -> Result<Option<Job>, Error> {
        Ok(self
            .db
            .get(id.to_be_bytes())?
            .map(|k| bincode::deserialize::<Job>(&k).unwrap()))
    }

    // we decrease the time till the job until there is a place in the database
    // as only one job can fire at the time, after a job gets a timeslot
    // it is never changed
    /// return the key for the job
    async fn add_job(&self, to_add: Job) -> Result<i64, Error> {
        let mut timestamp = to_add.time.timestamp().as_millisecond();
        let job = bincode::serialize(&to_add).unwrap();

        // create job entry if there is no job at this timestamp yet
        // if there is already a job scheduled, change the key for this one
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

    fn remove_job(&self, to_remove: i64) -> Result<Option<Job>, Error> {
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
                error!("Could not retrieve next job: {:?}", error);
                None
            }
        }
    }
}

fn event_timer(
    mut job_list: JobList,
    event_tx: broadcast::Sender<Event>,
    job_change_rx: mpsc::Receiver<()>,
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
            match job_change_rx.recv_timeout(timeout) {
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
            //no job to wait on, wait for instructions
            info!("no job in the future");
            //A message through the mpsc signals a job has been added
            match job_change_rx.recv() {
                // jobs were added or removed, go back and start waiting on them
                Ok(_) => break,
                // can't have timed out thus program should exit
                Err(_) => return,
            }
        }
    }
}
