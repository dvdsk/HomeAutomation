use mpd::{idle::Idle, idle::Subsystem, Client};
use crate::errors::Error;
use std::thread;
use std::sync::{Arc, RwLock};
use crossbeam_channel::{Receiver, Sender};

//mpd watcher, waits on mpd sys changes and stores in struct/sends updates to whomever needs to know
//mpd command sender (opens new connection for every command?)

#[derive(Clone)]
pub struct MpdStatus {
    inner: Arc<RwLock<mpd::status::Status>>,
}

impl MpdStatus {
    pub fn get_volume(&mut self) -> i8 {
        self.inner.read().unwrap().volume
    }

    pub fn is_playing(&mut self) -> mpd::status::State {
        self.inner.read().unwrap().state
    }
}

fn mpd_watcher(mut client: Client, status: MpdStatus, rx: Receiver<()>) {
    client.wait(&[Subsystem::Player, Subsystem::Mixer]).unwrap();
    loop {
        //if we have a message that means we should shut down
        if let Ok(_) = rx.try_recv(){
            return;
        }
        //this loop is stopped by sending a change to mpd server
        if let Ok(_) = client.wait(&[Subsystem::Player, Subsystem::Mixer]){
            //update status
            let new_status = client.status().unwrap();
            *(status.inner.write().unwrap()) = new_status;
        } else {
            return;
        }
    }
}

impl MpdStatus {
    pub fn stop_updating(tx: Sender<()>) {
        let mut client = mpd::Client::connect("127.0.0.1:6600").unwrap();
        tx.send(()).unwrap(); //tell mpd watcher to shut down on next check
        //make a small change to a subsys that is watched to force mpd_watcher to stop idle mode
        let status = client.status().unwrap();
        client.volume(status.volume+1).unwrap();
        client.volume(status.volume).unwrap();
    }

    pub fn start_updating() -> Result<(Self, thread::JoinHandle<()>, Sender<()>), Error>{
        let mut client = mpd::Client::connect("127.0.0.1:6600")?;
        let status = client.status()?;

        let mpd_status = Self { inner: Arc::new(RwLock::new(status))};
        let mpd_status_cloned = mpd_status.clone();
        let (tx, rx) = crossbeam_channel::bounded(1);

        let mpd_watcher_thread = thread::spawn(move || {
            mpd_watcher(client, mpd_status_cloned, rx);
        });

        Ok((mpd_status, mpd_watcher_thread, tx))
    }
}