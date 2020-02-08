use crossbeam_channel;
use std::thread;

use chrono::{DateTime, NaiveDateTime, Utc};
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
    db: sled::Tree,
}

fn waker(mut alarm_list: AlarmList, event_tx: crossbeam_channel::Sender<Event>, waker_rx: crossbeam_channel::Receiver<()>) {
    loop { 
        //This can fail #TODO make sure an non waking error alarm is send to the user
        if let Some(current_alarm) = alarm_list.get_next() {
            let now = Utc::now();
            let timeout = (current_alarm - now ).to_std().unwrap();//TODO handle alarm in past
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
        alarm_db.remove_old()?;

        let (waker_tx, waker_rx) = crossbeam_channel::unbounded();
        let waker_db_copy = alarm_db.clone();
        let waker_thread = thread::spawn(move || { waker(waker_db_copy, event_tx, waker_rx)});

        Ok((Self {alarm_db, waker_tx}, waker_thread))
    }

    // we decrease the time till the alarm until there is a place in the database
    // as only one alarm can fire at the time, after an alarm gets a timeslot it is never changed
    pub fn add_alarm(&self, at_time: DateTime<Utc>) -> Result<(), Error> {      
        self.alarm_db.add_alarm(at_time)?;
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
        let start: &[u8] = &[0];
        let alarms = self.alarm_db.db.range(start..);

        let mut list = Vec::new(); //TODO placeholder
        for (timestamp, _events) in alarms.filter_map(Result::ok) {
            let timestamp = BigEndian::read_u64(&timestamp) as i64;
            let alarm = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp, 0), Utc);
            list.push(alarm);
        }
        list
    }
}

impl AlarmList {
    // we decrease the time till the alarm until there is a place in the database
    // as only one alarm can fire at the time, after an alarm gets a timeslot it is never changed
    fn add_alarm(&self, at_time: DateTime<Utc>) -> Result<(), Error> {
        let mut timestamp = at_time.timestamp() as u64;
        let mut timestamp_array = timestamp.to_be_bytes();

        //create alarm entry if there is no alarm at this timestamp yet
        //if there is already an alarm schedualed, postpone this one until there is a spot free
        //self.db.cas(&[1], None as Option<&[u8]>, Some(&[10])
        //TODO check how this works and expand sled documentation for fail cases
        while let Err(_old_event) = self
            .db.compare_and_swap(&timestamp_array, None as Option<&[u8]>, Some(&[0]))? {//cas unique creation
            timestamp -= 1;
            timestamp_array = timestamp.to_be_bytes();
        }
        Ok(())
    }
    pub fn remove_alarm(&self, at_time: DateTime<Utc>) -> Result<(), Error> {
        let timestamp = at_time.timestamp() as u64;
        self.db.remove(timestamp.to_be_bytes())?;
        Ok(())
    }

    fn remove_old(&mut self) -> Result<(), Error> {
        let now = Utc::now();
        let timestamp = now.timestamp() as u64;
        let timestamp_array = timestamp.to_be_bytes();
        for old_alarm in self.db.range(..timestamp_array).keys() {
            if let Ok(old_alarm) = old_alarm {
                self.db.remove(old_alarm)?;
            }
        }
        Ok(())
    }
    /// calculate time to the earliest alarm, remove it from the list if the current time is later
    /// then the alarm
    fn get_next(&mut self) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        
        //the greater then is applied to the integer representation
        let timestamp = (now.timestamp()-10) as u64;
        match self.db.get_gt(timestamp.to_be_bytes()) {
            Ok(entry) => {
                if let Some(entry) = entry {
                    let (timestamp, _action) = entry;
                    let timestamp = BigEndian::read_u64(&timestamp) as i64;
                    let alarm = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp, 0), Utc);
                    Some(alarm)
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