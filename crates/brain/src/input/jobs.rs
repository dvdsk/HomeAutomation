use std::{
    fmt, sync::{mpsc, Arc}, thread, time::Duration
};

use jiff::{Span, ToSpan, Zoned};
use mpsc::RecvTimeoutError::*;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex};
use tracing::{error, info};

use crate::{controller::Event, time::to_next_datetime};

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("Could not store/edit job on disk")]
    DbError(#[from] sled::Error),
    #[error("Could not inform event timer about new job")]
    CommError(#[from] mpsc::SendError<()>),
    #[error("Dbstruct error")]
    DbStructError(#[from] dbstruct::Error<sled::Error>),
    // #[error("Dbstruct error")]
    // DbStructDoubleError(#[from] dbstruct::Error<dbstruct::Error<sled::Error>>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct Job {
    pub(crate) time: Zoned,
    pub(crate) event: Event,
    pub(crate) every_day: bool,
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
            time: to_next_datetime(hour, min),
            every_day: false,
            event,
            expiration,
        }
    }

    pub(crate) fn every_day_at(
        hour: i8,
        min: i8,
        event: Event,
        expiration: Option<Duration>,
    ) -> Job {
        Job {
            time: to_next_datetime(hour, min),
            every_day: true,
            event,
            expiration,
        }
    }

    pub(crate) fn add_one_day(mut self) -> Self {
        self.time = self.time.checked_add(1.day()).unwrap();
        self
    }
}

#[derive(Clone)]
pub(crate) struct Jobs {
    job_change_tx: mpsc::Sender<()>,
    list: Arc<Mutex<JobList>>,
}

impl fmt::Debug for Jobs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Jobs db and manager")
    }
}

#[dbstruct::dbstruct(db=sled)]
struct JobList {
    jobs: HashMap<i64, Job>,
}

impl Jobs {
    pub(crate) fn setup(
        event_tx: broadcast::Sender<Event>,
        db_path: &str,
    ) -> Result<Self, Error> {
        let job_list = Arc::new(Mutex::new(JobList::new(db_path)?));

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
    pub(crate) async fn add(&self, to_add: Job) -> Result<i64, Error> {
        let id = self.list.lock().await.add_job(to_add).await?;
        //signal event timer to update its next job
        self.job_change_tx.send(())?;
        Ok(id)
    }
    pub(crate) async fn remove(
        &self,
        to_remove: i64,
    ) -> Result<Option<Job>, Error> {
        let removed_job = self.list.lock().await.remove_job(to_remove)?;
        //signal event timer to update its next job
        self.job_change_tx.send(())?;
        Ok(removed_job)
    }
    pub(crate) async fn get(&self, id: i64) -> Result<Option<Job>, Error> {
        self.list.lock().await.get_job(id)
    }
}

impl JobList {
    fn get_job(&self, id: i64) -> Result<Option<Job>, Error> {
        Ok(self.jobs().get(&id)?)
    }

    // we decrease the id for the job until there is a place in the database
    // after a job gets an id it is never changed
    /// return the id for the job
    async fn add_job(&self, new_job: Job) -> Result<i64, Error> {
        let mut new_id = new_job.time.timestamp().as_millisecond();

        // create job entry if there is no job at this timestamp yet
        // if there is already a job scheduled, and it is not exactly the same,
        // change the id for this one until there is a spot free
        while let Some(old_job) = self.jobs().get(&new_id)? {
            if old_job == new_job {
                return Ok(new_id)
            }
            // create unique key
            new_id -= 1;
        }
        self.jobs().insert(&new_id, &new_job)?;
        Ok(new_id)
    }

    fn remove_job(&self, to_remove: i64) -> Result<Option<Job>, Error> {
        let old_job = self.jobs().remove(&to_remove)?;
        Ok(old_job)
    }

    fn peek_next(&mut self) -> Option<(i64, Job)> {
        let list: Vec<_> = self.jobs().iter().filter_map(|r| r.ok()).collect();
        list.get(0).cloned()
    }
}

async fn event_timer(
    job_list: Arc<Mutex<JobList>>,
    event_tx: broadcast::Sender<Event>,
    job_change_rx: mpsc::Receiver<()>,
) {
    loop {
        // This can fail
        // TODO make sure an non waking error alarm is send to the user
        if let Some((id, current_job)) = job_list.lock().await.peek_next() {
            let now = crate::time::now();
            let timeout = &current_job.time - &now;
            if let Some(expiration) = current_job.expiration {
                if now > &current_job.time + Span::try_from(expiration).unwrap()
                {
                    error!("skipping job too far in the past");
                    job_list.lock().await.remove_job(id).unwrap();
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
                        .send(current_job.event.clone())
                        .expect("controller should listen on this");
                    if current_job.every_day {
                        job_list
                            .lock()
                            .await
                            .add_job(current_job.add_one_day())
                            .await
                            .unwrap();
                    }
                    job_list.lock().await.remove_job(id).unwrap();
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
