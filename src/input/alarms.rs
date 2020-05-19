use crossbeam_channel;
use std::thread;
use std::ops::Sub;

use chrono::{DateTime, NaiveDateTime, Utc, self};
use sled;
use byteorder::{ByteOrder, BigEndian, ReadBytesExt};
use serde::{Serialize, Deserialize};

use crate::controller::Event;
use crate::controller::Command;
use crate::errors::Error;

//TODO, what to do on multiple alarms at the same time?
// -add one a second later
// -store in millisec to lower collision chance?

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Event(Event),
    Command(Command),
}

impl From<Event> for Action {
    fn from(event: Event) -> Self {
        Action::Event(event)
    }
}
impl From<Command> for Action {
    fn from(command: Command) -> Self {
        Action::Command(command)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Alarm {
    pub time: DateTime<Utc>,
    pub action: Action,
    pub expiration: Option<std::time::Duration>,
}

impl Alarm {
    pub fn from(time: DateTime<Utc>, action: impl Into<Action>, expiration: Option<std::time::Duration>)
     -> Self {
        Alarm {
            time,
            action: action.into(),
            expiration,
        }
    }
}

impl Sub<Alarm> for Alarm {
    type Output = chrono::Duration;
    
    fn sub(self, other: Alarm) -> chrono::Duration {
        self.time - other.time
    }
}
impl Sub<chrono::DateTime<Utc>> for &Alarm {
    type Output = chrono::Duration;
    
    fn sub(self, other: chrono::DateTime<Utc>) -> chrono::Duration {
        self.time - other
    }
}

#[derive(Clone)]
pub struct Alarms {
    waker_tx: crossbeam_channel::Sender<()>,
    alarm_db: AlarmList,
}

#[derive(Clone)]
pub struct AlarmList {
    db: sled::Tree,
}

fn waker(mut alarm_list: AlarmList, event_tx: crossbeam_channel::Sender<Event>, waker_rx: crossbeam_channel::Receiver<()>) {
    loop { 
        //This can fail #TODO make sure an non waking error alarm is send to the user
        if let Some((id, current_alarm)) = alarm_list.get_next() {
            let now = Utc::now();
            let timeout = &current_alarm - now;
            if let Some(expiration) = current_alarm.expiration {
                if timeout < -chrono::Duration::from_std(expiration).unwrap() {
                    error!("skipping alarm to far in the past");
                    alarm_list.remove_alarm(id).unwrap();
                    continue; //alarm to far in the past, skip and get next
                }
            }
            let timeout = timeout.to_std()
                .unwrap_or(std::time::Duration::from_secs(0));
            info!("next alarm is in: {} seconds", timeout.as_secs());

            //do we sound the an alarm or should we add or remove one?
            match waker_rx.recv_timeout(timeout) {
                //do not set off alarm
                Ok(_) => {dbg!(); ()},//should recheck if "current alarm" is still the right one as we removed one
                    
                
                Err(error) => match error {//should the alarm go off or should we stop?
                    crossbeam_channel::RecvTimeoutError::Timeout => {
                        if let Err(error) = event_tx.send(Event::Alarm){ 
                            error!("could not set off alarm: {:?}", error);
                        }; //set 
                        //remove alarm from memory and file
                        alarm_list.remove_alarm(id).unwrap();
                        continue; //get next alarm
                    }
                    //we should stop, end the thread by returning
                    crossbeam_channel::RecvTimeoutError::Disconnected => return,                       
                }
            }
        } else {
            //no alarm to wait on, wait for instructions
            info!("no alarm in the future");
            loop {
                //A message through the mpsc signals an alarm has been added
                match waker_rx.recv() {
                    Ok(_) => break, //alarms were added or remove, go back and start waiting on it
                    Err(_) => return, //cant have timed out thus program should exit
                }
            }
        }

    }
}

impl Alarms {

    pub fn setup(event_tx: crossbeam_channel::Sender<Event>, db: sled::Db) -> Result<(Self, thread::JoinHandle<()>), Error> {

        let mut alarm_db = AlarmList { db: db.open_tree("alarms")? };

        let (waker_tx, waker_rx) = crossbeam_channel::unbounded();
        let waker_db_copy = alarm_db.clone();
        let waker_thread = thread::spawn(move || { waker(waker_db_copy, event_tx, waker_rx)});

        Ok((Self {alarm_db, waker_tx}, waker_thread))
    }

    // we decrease the time till the alarm until there is a place in the database
    // as only one alarm can fire at the time, after an alarm gets a timeslot it is never changed
    pub async fn add_alarm(&self, to_add: Alarm) -> Result<(), Error> {      
        self.alarm_db.add_alarm(to_add).await?;
        //signal waker to update its next alarm
        self.waker_tx.send(())?;
        Ok(())
    }
    pub fn remove_alarm(&self, to_remove: u64) -> Result<Option<Alarm>, Error> {
        let removed_alarm = self.alarm_db.remove_alarm(to_remove)?;
        self.waker_tx.send(())?; //signal waker to update its next alarm
        Ok(removed_alarm)
    }

    pub fn list(&self) -> Vec<(u64, Alarm)> {
        //self.alarm_list.iter().keys().map(|k| DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(k, 0), Utc);)
        let start: &[u8] = &[0];
        let alarms = self.alarm_db.db.range(start..);

        let mut list = Vec::new(); //TODO placeholder
        for (key, alarm) in alarms.filter_map(Result::ok) {
            let alarm = bincode::deserialize(&alarm).unwrap();
            let key = key.as_ref().read_u64::<BigEndian>().unwrap();
            list.push((key,alarm));
        }
        list
    }
}

impl AlarmList {
    // we decrease the time till the alarm until there is a place in the database
    // as only one alarm can fire at the time, after an alarm gets a timeslot it is never changed
    async fn add_alarm(&self, to_add: Alarm) -> Result<(), Error> {
        let mut timestamp = to_add.time.timestamp() as u64;
        let mut timestamp_array = timestamp.to_be_bytes();
        let mut alarm = bincode::serialize(&to_add).unwrap();

        //create alarm entry if there is no alarm at this timestamp yet
        //if there is already an alarm schedualed, postpone this one until there is a spot free
        //self.db.cas(&[1], None as Option<&[u8]>, Some(&[10])
        //TODO check how this works and expand sled documentation for fail cases
        while let Err(_old_event) = self
            .db
            .compare_and_swap(&timestamp_array, None as Option<&[u8]>, Some(alarm.clone()))? {
            //cas unique creation
            timestamp -= 1;
            timestamp_array = timestamp.to_be_bytes();
        }
        self.db.flush_async().await?;
        Ok(())
    }
    pub fn remove_alarm(&self, to_remove: u64) -> Result<Option<Alarm>, Error> {
        Ok(self.db
            .remove(to_remove.to_be_bytes())?
            .map(|k| bincode::deserialize::<Alarm>(&k).unwrap()))
    }

    /// calculate time to the earliest alarm, remove it from the list if the current time is later
    /// then the alarm
    fn get_next(&mut self) -> Option<(u64,Alarm)> {
        let now = Utc::now();
        
        //get earliest alarm time in db
        match self.db.get_gt(0u64.to_be_bytes()) {
            Ok(entry) => {
                if let Some(entry) = entry {
                    let (id, alarm) = entry;
                    let id = id.as_ref().read_u64::<BigEndian>().unwrap();
                    let alarm = bincode::deserialize(&alarm).unwrap();
                    Some((id,alarm))
                } else { 
                    None
                }
            },
            Err(error) => {
                error!("Could not retrieve next alarm: {:?}",error);
                None
            }
        }
    }
}