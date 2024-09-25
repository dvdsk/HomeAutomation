use std::ops::Sub;
use std::sync::mpsc;
use std::thread;
use tokio::sync::broadcast;
use tracing::error;
use tracing::info;

use byteorder::{BigEndian, ReadBytesExt};
use chrono::{self, DateTime, Utc};
use serde::{Deserialize, Serialize};
use sled;

use crate::controller::Event;

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
    pub time: DateTime<Utc>,
    pub event: Event,
    /// how long after the time was missed the alarm
    /// should still go off
    pub expiration: Option<std::time::Duration>,
}

impl Job {
    pub fn from(
        time: DateTime<Utc>,
        event: Event,
        expiration: Option<std::time::Duration>,
    ) -> Self {
        Job {
            time,
            event,
            expiration,
        }
    }
}

impl Sub<Job> for Job {
    type Output = chrono::Duration;

    fn sub(self, other: Job) -> chrono::Duration {
        self.time - other.time
    }
}
impl Sub<chrono::DateTime<Utc>> for &Job {
    type Output = chrono::Duration;

    fn sub(self, other: chrono::DateTime<Utc>) -> chrono::Duration {
        self.time - other
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
    mut alarm_list: JobList,
    event_tx: broadcast::Sender<Event>,
    waker_rx: mpsc::Receiver<()>,
) {
    loop {
        //This can fail #TODO make sure an non waking error alarm is send to the user
        if let Some((id, current_alarm)) = alarm_list.peek_next() {
            let now = Utc::now();
            let timeout = &current_alarm - now;
            if let Some(expiration) = current_alarm.expiration {
                if timeout < -chrono::Duration::from_std(expiration).unwrap() {
                    error!("skipping alarm to far in the past");
                    alarm_list.remove_alarm(id).unwrap();
                    continue; //alarm to far in the past, skip and get next
                }
            }
            let timeout = timeout
                .to_std()
                .unwrap_or(std::time::Duration::from_secs(0));
            info!("next alarm is in: {} seconds", timeout.as_secs());

            //do we sound an alarm or should we add or remove one?
            match waker_rx.recv_timeout(timeout) {
                Ok(_) => continue, // new alarm entered restart loop
                Err(Disconnected) => return,
                Err(Timeout) => {
                    // time to sound the alarm
                    event_tx
                        .send(current_alarm.event)
                        .expect("controller should listen on this");
                    alarm_list.remove_alarm(id).unwrap();
                    continue; //get next alarm
                }
            }
        } else {
            //no alarm to wait on, wait for instructions
            info!("no alarm in the future");
            //A message through the mpsc signals an alarm has been added
            match waker_rx.recv() {
                Ok(_) => break,   //alarms were added or remove, go back and start waiting on it
                Err(_) => return, //cant have timed out thus program should exit
            }
        }
    }
}

impl Jobs {
    pub fn setup(
        event_tx: broadcast::Sender<Event>,
        db: sled::Db,
    ) -> Result<(Self, thread::JoinHandle<()>), Error> {
        let list = JobList {
            db: db.open_tree("alarms")?,
        };

        let (waker_tx, waker_rx) = mpsc::channel();
        let waker_db_copy = list.clone();
        let waker_thread = thread::spawn(move || waker(waker_db_copy, event_tx, waker_rx));

        Ok((Self { list, waker_tx }, waker_thread))
    }

    // we decrease the time till the alarm until there is a place in the database
    // as only one alarm can fire at the time, after an alarm gets a timeslot it is never changed
    pub async fn add_alarm(&self, to_add: Job) -> Result<u64, Error> {
        let id = self.list.add_alarm(to_add).await?;
        //signal waker to update its next alarm
        self.waker_tx.send(())?;
        Ok(id)
    }
    pub fn remove_alarm(&self, to_remove: u64) -> Result<Option<Job>, Error> {
        let removed_alarm = self.list.remove_alarm(to_remove)?;
        self.waker_tx.send(())?; //signal waker to update its next alarm
        Ok(removed_alarm)
    }
    pub fn get(&self, id: u64) -> Result<Option<Job>, Error> {
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
    pub fn get_alarm(&self, id: u64) -> Result<Option<Job>, Error> {
        Ok(self
            .db
            .get(id.to_be_bytes())?
            .map(|k| bincode::deserialize::<Job>(&k).unwrap()))
    }

    // we decrease the time till the alarm until there is a place in the database
    // as only one alarm can fire at the time, after an alarm gets a timeslot it is never changed
    /// return the key for the alarm
    async fn add_alarm(&self, to_add: Job) -> Result<u64, Error> {
        let mut timestamp = to_add.time.timestamp() as u64;
        let mut timestamp_array = timestamp.to_be_bytes();
        let alarm = bincode::serialize(&to_add).unwrap();

        //create alarm entry if there is no alarm at this timestamp yet
        //if there is already an alarm scheduled, postpone this one until there is a spot free
        //self.db.cas(&[1], None as Option<&[u8]>, Some(&[10])
        //TODO check how this works and expand sled documentation for fail cases
        while let Err(_old_event) = self.db.compare_and_swap(
            timestamp_array,
            None as Option<&[u8]>,
            Some(alarm.clone()),
        )? {
            //cas unique creation
            timestamp -= 1;
            timestamp_array = timestamp.to_be_bytes();
        }
        self.db.flush_async().await?;
        Ok(timestamp)
    }

    pub fn remove_alarm(&self, to_remove: u64) -> Result<Option<Job>, Error> {
        let old_alarm = self
            .db
            .remove(to_remove.to_be_bytes())?
            .map(|k| bincode::deserialize::<Job>(&k).unwrap());
        self.db.flush()?;
        Ok(old_alarm)
    }

    /// calculate time to the earliest alarm, remove it from the list if the current time is later
    /// then the alarm
    fn peek_next(&mut self) -> Option<(u64, Job)> {
        //get earliest alarm time in db
        match self.db.get_gt(0u64.to_be_bytes()) {
            Ok(entry) => {
                if let Some(entry) = entry {
                    let (id, alarm) = entry;
                    let id = id.as_ref().read_u64::<BigEndian>().unwrap();
                    let alarm = bincode::deserialize(&alarm).unwrap();
                    Some((id, alarm))
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
