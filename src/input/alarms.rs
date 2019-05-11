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

struct AlarmList {
    memory: BTreeSet<DateTime<Utc>>,
    file: File,
}

enum WakerMessage {
    RemoveAlarm(DateTime<Utc>),
    AddAlarm(DateTime<Utc>),
}

#[derive(Debug)]
struct Alarms {
    waker_tx: mpsc::Sender<WakerMessage>,
    waker_thread: thread::JoinHandle<()>,
}

fn waker(mut alarm_list: AlarmList, event_tx: mpsc::Sender<Event>, waker_rx: mpsc::Receiver<WakerMessage>) {
    
    loop { 
        if let Some(current_alarm) = alarm_list.get_next_alarm() {
            loop { //we have an alarm time, keep waiting until is should go off, handle instructions in the mean time
                let now = Utc::now();
                let timeout = (now - current_alarm).to_std().unwrap();
                
                //do we sound the an alarm or should we add or remove one?
                match waker_rx.recv_timeout(timeout) {
                    Ok(message) => match message {//the alarm should not go off
                        WakerMessage::RemoveAlarm(at_time) => alarm_list.remove_alarm(&at_time),
                        WakerMessage::AddAlarm(at_time) => alarm_list.add_alarm(at_time),
                    },
                    Err(error) => match error {//should the alarm go off or should we stop?
                        mpsc::RecvTimeoutError::Timeout => {
                            event_tx.send(Event::Alarm); //set 
                            alarm_list.remove_alarm(&current_alarm);
                            break; //get next alarm
                        }
                        //we should stop, end the thread by returning
                        mpsc::RecvTimeoutError::Disconnected => return,                       
                    }
                }
            }
        } else {
            //we have no alarm to wait on, wait for instructions
            loop {
                match waker_rx.recv() {
                    Ok(message) => match message {
                        WakerMessage::RemoveAlarm(at_time) => alarm_list.remove_alarm(&at_time),
                        //should not only add the alarm but go back and start waiting on it
                        WakerMessage::AddAlarm(at_time) => { alarm_list.add_alarm(at_time); break },
                    }
                    Err(_) => return //cant have timed out thus program should exit
                }
            }
        }

    }
}

impl Alarms {

    fn setup_alarms(event_tx: mpsc::Sender<Event>) -> Result<Self, Error> {
        let mut alarm_list = AlarmList::load()?;
    
        let (waker_tx, waker_rx) = mpsc::channel();
        let waker_thread = thread::spawn(move || { waker(alarm_list, event_tx, waker_rx)});

        Ok(Self {waker_tx, waker_thread})
    }

    fn add_alarm(&mut self, at_time: DateTime<Utc>){
        self.waker_tx.send(WakerMessage::AddAlarm(at_time));
    }
    fn remove_alarm(&mut self, at_time: DateTime<Utc>){
        self.waker_tx.send(WakerMessage::RemoveAlarm(at_time));
    }
}

impl AlarmList {

    fn load() -> Result<Self, Error> {
        let path = Path::new("alarms.yaml");
        
        let mut file;
        let memory;
        if path.exists() {
            file = File::open(path)?;
            memory = serde_yaml::from_reader(&mut file)?
        } else {
            file = File::create(path)?;
            memory = BTreeSet::new();
        }
        Ok(Self {memory, file })
    }

    fn add_alarm(&mut self, at_time: DateTime<Utc>) {
        self.memory.insert(at_time);
        self.file.set_len(0); //truncate file
        serde_yaml::to_writer(&mut self.file, &self.memory);
    }
    fn remove_alarm(&mut self, at_time: &DateTime<Utc>) {
        self.memory.remove(at_time);
        self.file.set_len(0); //truncate file
        serde_yaml::to_writer(&mut self.file, &self.memory);
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