use std::collections::BTreeSet;
use std::path::Path;

use crossbeam_channel;
use std::sync::{Arc, Mutex};
use std::thread;

use chrono::{DateTime, NaiveDateTime, Utc};
use std::fs::{File, OpenOptions};
use sled;
use byteorder::{ByteOrder, BigEndian};

use crate::controller::Event;
use crate::errors::Error;

//TODO, what to do on multiple alarms at the same time?
// -add one a second later
// -store in millisec to lower collision chance?




#[derive(Clone)]
pub struct Alarms {
    waker_tx: crossbeam_channel::Sender<()>,
    alarm_db: AlarmList,
}

#[derive(Clone)]
pub struct AlarmList {
    db: Arc<sled::Tree>,
}

fn waker(mut alarm_list: AlarmList, event_tx: crossbeam_channel::Sender<Event>, waker_rx: crossbeam_channel::Receiver<()>) {
    
    loop { 
        //This can fail #TODO make sure an non waking error alarm is send to the user
        if let Some(current_alarm) = alarm_list.get_next_alarm() {
            let now = Utc::now();
            let timeout = (now - current_alarm).to_std().unwrap();
            
            //do we sound the an alarm or should we add or remove one?
            match waker_rx.recv_timeout(timeout) {
                //do not set off alarm
                Ok(_) => (),//should recheck if "current alarm" is still the right one as we removed one
                    
                
                Err(error) => match error {//should the alarm go off or should we stop?
                    crossbeam_channel::RecvTimeoutError::Timeout => {
                        if let Err(error) = event_tx.send(Event::Alarm){ 
                            error!("could not set off alarm: {:?}", error);
                        }; //set 
                        //remove alarm from memory and file
                        if let Err(error) = alarm_list.remove_alarm(current_alarm){
                            error!("could not remove alarm after its trigger time: {:?}", error);
                        };
                        break; //get next alarm
                    }
                    //we should stop, end the thread by returning
                    crossbeam_channel::RecvTimeoutError::Disconnected => return,                       
                }
            }
        } else {
            //no alarm to wait on, wait for instructions
            loop {
                //A message through the mpsc signals an alarm has been added
                match waker_rx.recv() {
                    Ok(_) => (), //alarms were added or remove, go back and start waiting on it
                    Err(_) => return //cant have timed out thus program should exit
                }
            }
        }

    }
}

impl Alarms {

    pub fn setup(event_tx: crossbeam_channel::Sender<Event>, db: sled::Db) -> Result<(Self, thread::JoinHandle<()>), Error> {

        let alarm_db = AlarmList { db: db.open_tree("alarms")? };

        let (waker_tx, waker_rx) = crossbeam_channel::unbounded();
        let waker_db_copy = alarm_db.clone();
        let waker_thread = thread::spawn(move || { waker(waker_db_copy, event_tx, waker_rx)});

        Ok((Self {alarm_db, waker_tx}, waker_thread))
    }

    // we decrease the time till the alarm until there is a place in the database
    // as only one alarm can fire at the time, after an alarm gets a timeslot it is never changed
    pub fn add_alarm(&self, at_time: DateTime<Utc>) -> Result<(), Error> {
        let mut timestamp = at_time.timestamp();
        
        self.alarm_db.add_alarm(at_time);
        //signal waker to update its next alarm
        self.waker_tx.send(())?;
        Ok(())
    }
    pub fn remove_alarm(&self, at_time: DateTime<Utc>) -> Result<(), Error> {
        self.alarm_db.remove_alarm(at_time)?;
        self.waker_tx.send(())?; //signal waker to update its next alarm
        Ok(())
    }

    pub fn list(&self) -> Vec<DateTime<Utc>> {
        //self.alarm_list.iter().keys().map(|k| DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(k, 0), Utc);)
        Vec::new() //TODO placeholder
    }
}

impl AlarmList {
    // we decrease the time till the alarm until there is a place in the database
    // as only one alarm can fire at the time, after an alarm gets a timeslot it is never changed
    pub fn add_alarm(&self, at_time: DateTime<Utc>) -> Result<(), Error> {
        let mut timestamp = at_time.timestamp() as u64;
        let mut timestamp_array = timestamp.to_be_bytes();

        //create alarm entry if there is no alarm at this timestamp yet
        //if there is already an alarm schedualed, postpone this one until there is a spot free
        //self.db.cas(&[1], None as Option<&[u8]>, Some(&[10])
        while let Err(old_event) = self.db.cas(&timestamp_array, None as Option<&[u8]>, Some(&[0]))? {//cas unique creation
            timestamp -= 1;
            timestamp_array = timestamp.to_be_bytes();
        }
        Ok(())
    }
    pub fn remove_alarm(&self, at_time: DateTime<Utc>) -> Result<(), Error> {
        let timestamp = at_time.timestamp() as u64;
        self.db.del(timestamp.to_be_bytes())?;
        Ok(())
    }

    /// calculate time to the earliest alarm, remove it from the list if the current time is later
    /// then the alarm
    fn get_next_alarm(&mut self) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        
        //the greater then is applied to the integer representation
        let timestamp = (now.timestamp()-10) as u64;
        match self.db.get_gt(timestamp.to_be_bytes()) {
            Ok(entry) => {
                if let Some(entry) = entry {
                    let (timestamp, action) = entry;
                    let timestamp = BigEndian::read_u64(&timestamp);
                    let alarm = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc);
                    Some(alarm)
                } else { 
                    None
                }
            },
            Err(error) => {
                error!("Could not retrieve next alarm");
                None
            }
        }
    }
}