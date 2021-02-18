use crossbeam_channel;
use std::ops::Sub;
use std::thread;

use byteorder::{BigEndian, ReadBytesExt};
use chrono::{self, DateTime, Utc};
use serde::{Deserialize, Serialize};
use sled;

use crate::controller::Command;
use crate::controller::Event;
use crate::errors::Error;

mod wakeup;
pub use wakeup::WakeUp;

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    SendEvent(Event),
    SendCommand(Command),
}

impl From<Event> for Action {
    fn from(event: Event) -> Self {
        Action::SendEvent(event)
    }
}
impl From<Command> for Action {
    fn from(command: Command) -> Self {
        Action::SendCommand(command)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Job {
    pub time: DateTime<Utc>,
    pub action: Action,
    /// how long after the time was missed the alarm
    /// should still go off
    pub expiration: Option<std::time::Duration>,
}

impl Job {
    pub fn from(
        time: DateTime<Utc>,
        action: impl Into<Action>,
        expiration: Option<std::time::Duration>,
    ) -> Self {
        Job {
            time,
            action: action.into(),
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

#[derive(Clone)]
pub struct Jobs {
    waker_tx: crossbeam_channel::Sender<()>,
    list: JobList,
}

#[derive(Clone)]
pub struct JobList {
    db: sled::Tree,
}

use crossbeam_channel::RecvTimeoutError::*;
fn waker(
    mut alarm_list: JobList,
    event_tx: crossbeam_channel::Sender<Event>,
    waker_rx: crossbeam_channel::Receiver<()>,
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
                    sound_alarm(&mut alarm_list, &event_tx, current_alarm);
                    alarm_list.remove_alarm(id).unwrap();
                    continue; //get next alarm
                }
            }
        } else {
            //no alarm to wait on, wait for instructions
            info!("no alarm in the future");
            loop {
                //A message through the mpsc signals an alarm has been added
                match waker_rx.recv() {
                    Ok(_) => break,   //alarms were added or remove, go back and start waiting on it
                    Err(_) => return, //cant have timed out thus program should exit
                }
            }
        }
    }
}

fn sound_alarm(list: &mut JobList, event_tx: &crossbeam_channel::Sender<Event>, job: Job) {
    match job.action {
        Action::SendEvent(ev) => event_tx.send(ev).unwrap(),
        Action::SendCommand(cmd) => event_tx.send(Event::Command(cmd)).unwrap(),
    }
}

impl Jobs {
    pub fn setup(
        event_tx: crossbeam_channel::Sender<Event>,
        db: sled::Db,
    ) -> Result<(Self, thread::JoinHandle<()>), Error> {
        let list = JobList {
            db: db.open_tree("alarms")?,
        };

        let (waker_tx, waker_rx) = crossbeam_channel::unbounded();
        let waker_db_copy = list.clone();
        let waker_thread = thread::spawn(move || waker(waker_db_copy, event_tx, waker_rx));

        Ok((Self { list, waker_tx }, waker_thread))
    }

    // we decrease the time till the alarm until there is a place in the database
    // as only one alarm can fire at the time, after an alarm gets a timeslot it is never changed
    pub async fn add_alarm(&self, to_add: Job) -> Result<(), Error> {
        self.list.add_alarm(to_add).await?;
        //signal waker to update its next alarm
        self.waker_tx.send(())?;
        Ok(())
    }
    pub fn remove_alarm(&self, to_remove: u64) -> Result<Option<Job>, Error> {
        let removed_alarm = self.list.remove_alarm(to_remove)?;
        self.waker_tx.send(())?; //signal waker to update its next alarm
        Ok(removed_alarm)
    }

    pub fn list(&self) -> Vec<(u64, Job)> {
        //self.alarm_list.iter().keys().map(|k| DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(k, 0), Utc);)
        let start: &[u8] = &[0];
        let alarms = self.list.db.range(start..);

        let mut list = Vec::new(); //TODO placeholder
        for (key, alarm) in alarms.filter_map(Result::ok) {
            let alarm = bincode::deserialize(&alarm).unwrap();
            let key = key.as_ref().read_u64::<BigEndian>().unwrap();
            list.push((key, alarm));
        }
        list
    }
}

impl JobList {
    // we decrease the time till the alarm until there is a place in the database
    // as only one alarm can fire at the time, after an alarm gets a timeslot it is never changed
    /// return the key for the alarm
    async fn add_alarm(&self, to_add: Job) -> Result<u64, Error> {
        let mut timestamp = to_add.time.timestamp() as u64;
        let mut timestamp_array = timestamp.to_be_bytes();
        let alarm = bincode::serialize(&to_add).unwrap();

        //create alarm entry if there is no alarm at this timestamp yet
        //if there is already an alarm schedualed, postpone this one until there is a spot free
        //self.db.cas(&[1], None as Option<&[u8]>, Some(&[10])
        //TODO check how this works and expand sled documentation for fail cases
        while let Err(_old_event) = self.db.compare_and_swap(
            &timestamp_array,
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
        Ok(self
            .db
            .remove(to_remove.to_be_bytes())?
            .map(|k| bincode::deserialize::<Job>(&k).unwrap()))
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
