use std::collections::BTreeSet;
use std::path::Path;

use crossbeam_channel;
use std::sync::{Arc, Mutex};
use std::thread;

use chrono::{DateTime, Utc};
use std::fs::{File, OpenOptions};
use serde_yaml;

use crate::controller::Event;
use crate::errors::Error;

//TODO, move alarms to input? then backup waker/trigger/speaker/alarm moves to arduino

#[derive(Debug)]
struct RawList {
    memory: BTreeSet<DateTime<Utc>>,
    file: File,   
}

#[derive(Debug, Clone)]
struct AlarmList {
    rawlist: Arc<Mutex<RawList>>,
}

#[derive(Debug, Clone)]
pub struct Alarms {
    waker_tx: crossbeam_channel::Sender<()>,
    alarm_list: AlarmList,
}

fn waker(mut alarm_list: AlarmList, event_tx: crossbeam_channel::Sender<Event>, waker_rx: crossbeam_channel::Receiver<()>) {
    
    loop { 
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
                        if let Err(error) = alarm_list.remove_alarm(&current_alarm){
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

    pub fn setup(event_tx: crossbeam_channel::Sender<Event>) -> Result<(Self, thread::JoinHandle<()>), Error> {
        let mut alarm_list = AlarmList::load()?;
        let (waker_tx, waker_rx) = crossbeam_channel::unbounded();
        let mut alarm_list_for_waker = alarm_list.clone();
        let waker_thread = thread::spawn(move || { waker(alarm_list_for_waker, event_tx, waker_rx)});

        Ok((Self {alarm_list, waker_tx}, waker_thread))
    }

    pub fn add_alarm(&self, at_time: DateTime<Utc>) -> Result<(), Error> {
        self.alarm_list.add_alarm(at_time)?;
        self.waker_tx.send(())?;
        Ok(())
    }
    pub fn remove_alarm(&self, at_time: DateTime<Utc>) -> Result<(), Error> {
        self.alarm_list.remove_alarm(&at_time)?;
        self.waker_tx.send(())?;
        Ok(())
    }

    pub fn list(&self) -> Vec<DateTime<Utc>> {
        self.alarm_list.list()
    }
}

impl AlarmList {

    fn load() -> Result<Self, Error> {
        let list = RawList::load()?;
        Ok(Self {rawlist: Arc::new(Mutex::new(list)) })
    }

    fn add_alarm(&self, at_time: DateTime<Utc>) -> Result<(), Error> {
        let mut list = self.rawlist.lock().unwrap();
        list.add_alarm(at_time)
    }
    fn remove_alarm(&self, at_time: &DateTime<Utc>) -> Result<(), Error> {
        let mut list = self.rawlist.lock().unwrap();
        list.remove_alarm(at_time)
    }

    fn list(&self) -> Vec<DateTime<Utc>> {
        let mut list = self.rawlist.lock().unwrap();
        list.memory.iter().cloned().collect()
    }

    /// calculate time to the earliest alarm, remove it from the list if the current time is later
    /// then the alarm
    fn get_next_alarm(&mut self) -> Option<DateTime<Utc>> {
        let mut list = self.rawlist.lock().unwrap();
        list.get_next_alarm()
    }
}

impl RawList {

    fn load() -> Result<Self, Error> {
        let path = Path::new("alarms.yaml");
        
        let mut file;
        let memory;
        if path.exists() {
            file = OpenOptions::new().write(true).read(true).open(path)?;
            memory = serde_yaml::from_reader(&mut file)?;
            info!("loaded alarms from file");
        } else {
            file = File::create(path)?;
            memory = BTreeSet::new();
            serde_yaml::to_writer(&file, &memory)?;
            info!("alarm file did not exist, created new");
        }

        Ok(Self {memory, file})
    }

    fn add_alarm(&mut self, at_time: DateTime<Utc>) -> Result<(), Error>{
        self.memory.insert(at_time);
        self.file.set_len(0)?; //truncate file
        serde_yaml::to_writer(&mut self.file, &self.memory)?;
        self.file.sync_data()?;
        Ok(())
    }
    fn remove_alarm(&mut self, at_time: &DateTime<Utc>) -> Result<(), Error>{
        self.memory.remove(at_time);
        self.file.set_len(0)?; //truncate file
        serde_yaml::to_writer(&mut self.file, &self.memory)?;
        self.file.sync_data()?;
        Ok(())
    }

    /// calculate time to the earliest alarm, remove it from the list if the current time is later
    /// then the alarm
    fn get_next_alarm(&mut self) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        
        loop {
            let next_alarm = self.memory.iter().cloned().next();
            if let Some(alarm) = next_alarm {
                if alarm > now {
                    return Some(alarm.clone());
                } else {
                    if self.remove_alarm(&alarm).is_err() {
                        error!("could not remove alarm after it fired!"); 
                    }
                }
            } else {
                return None;
            }
        } 
    }
}