use std::collections::BTreeSet;
use std::path::Path;

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::fs::File;
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
    waker_tx: mpsc::Sender<()>,
    alarm_list: AlarmList,
}

fn waker(mut alarm_list: AlarmList, event_tx: mpsc::Sender<Event>, waker_rx: mpsc::Receiver<()>) {
    
    loop { 
        if let Some(current_alarm) = alarm_list.get_next_alarm() {
            let now = Utc::now();
            let timeout = (now - current_alarm).to_std().unwrap();
            
            //do we sound the an alarm or should we add or remove one?
            match waker_rx.recv_timeout(timeout) {
                //do not set off alarm
                Ok(_) => (),//should recheck if "current alarm" is still the right one as we removed one
                    
                
                Err(error) => match error {//should the alarm go off or should we stop?
                    mpsc::RecvTimeoutError::Timeout => {
                        event_tx.send(Event::Alarm); //set 
                        //remove alarm from memory and file
                        alarm_list.remove_alarm(&current_alarm);
                        break; //get next alarm
                    }
                    //we should stop, end the thread by returning
                    mpsc::RecvTimeoutError::Disconnected => return,                       
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

    pub fn setup(event_tx: mpsc::Sender<Event>) -> Result<(Self, thread::JoinHandle<()>), Error> {
        dbg!(("hoi"));
        let mut alarm_list = AlarmList::load()?;
        let (waker_tx, waker_rx) = mpsc::channel();
        let mut alarm_list_for_waker = alarm_list.clone();
        let waker_thread = thread::spawn(move || { waker(alarm_list_for_waker, event_tx, waker_rx)});

        Ok((Self {alarm_list, waker_tx}, waker_thread))
    }

    pub fn add_alarm(&self, at_time: DateTime<Utc>) -> Result<(), Error> {
        self.alarm_list.remove_alarm(&at_time)?;
        self.waker_tx.send(());
        Ok(())
    }
    pub fn remove_alarm(&self, at_time: DateTime<Utc>) -> Result<(), Error> {
        self.alarm_list.remove_alarm(&at_time)?;
        self.waker_tx.send(());
        Ok(())
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
            file = File::open(path)?;
            memory = serde_yaml::from_reader(&mut file)?;
        } else {
            file = File::create(path)?;
            memory = BTreeSet::new();
            serde_yaml::to_writer(&file, &memory)?;
        }

        Ok(Self {memory, file})
    }

    fn add_alarm(&mut self, at_time: DateTime<Utc>) -> Result<(), Error>{
        self.memory.insert(at_time);
        self.file.set_len(0)?; //truncate file
        serde_yaml::to_writer(&mut self.file, &self.memory)?;
        Ok(())
    }
    fn remove_alarm(&mut self, at_time: &DateTime<Utc>) -> Result<(), Error>{
        self.memory.remove(at_time);
        self.file.set_len(0)?; //truncate file
        serde_yaml::to_writer(&mut self.file, &self.memory)?;
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
                    self.remove_alarm(&alarm);
                }
            } else {
                return None;
            }
        } 
    }
}