use gpio_cdev::{Chip, LineRequestFlags, EventRequestFlags, LineEventHandle};
use nix::poll::{PollFd, PollFlags, poll};
use std::os::unix::io::{AsRawFd};
use smallvec::{SmallVec};
use log::error;

use std::thread;
use crossbeam_channel;

use crate::errors::Error;
use crate::controller::{Command, Event};

// when pressing 22, 24 is often activated
// 23 works perfectly
// 24 also works perfectly
const MILLIS: u64 = 1_000_000; //nano to milli 
const MAX_TAP_LEN: u64 = 600 * MILLIS;
const N_LINES: usize = 54;

fn detect_and_handle(chip: &mut Chip, tx: crossbeam_channel::Sender<Event>)
 -> Result<(), Error>{

    let offsets: [u32; 10] = [27,22,18, 23,24,26,17, 16,12,13];   
    let (mut evt_handles, mut pollables) = configure_watching(chip, &offsets)?;
    thread::spawn(move || { 
        let mut last_high = [0u64; N_LINES];
        let mut last_state = [0u8; N_LINES];
        
        loop{
            if poll(&mut pollables, -1).unwrap() !=0 {
                let key_presses = process_event(&pollables, 
                    &mut evt_handles, &mut last_high, &mut last_state);
                for (offset, down_duration) in key_presses {
                    if down_duration > 10*MILLIS { //debounce
                        if let Some(event) = to_command(offset, down_duration){
                            tx.send(event).unwrap();
                        }
                    }
                }
            }
        }
    });
    Ok(())
}

///returns keys that where held as the time they where held in nanoseconds
fn process_event(pollables: &Vec<PollFd>, evt_handles: &mut Vec<LineEventHandle>,
    last_rising: &mut [u64], last_state: &mut [u8])
     -> SmallVec::<[(usize,u64); 64]> {
    
    let mut key_presses = SmallVec::<[(usize,u64); 64]>::new();
    for i in 0..pollables.len() {
        if let Some(poll_res) = pollables[i].revents() {
            let h = &mut evt_handles[i];
            if poll_res.contains(PollFlags::POLLIN) {
                let value = h.get_value().unwrap();
                let event = h.get_event().unwrap();
                let offset = h.line().offset() as usize;
                
                if value == 1 && last_state[offset] == 0 {
                    //rising
                    last_state[offset] = 1;
                    last_rising[offset] = event.timestamp();
                } else if value == 0 && last_state[offset] == 1 {
                    //falling update current state and store duration of keypress
                    last_state[offset] = 0;
                    let held_for = event.timestamp()-last_rising[offset];
                    key_presses.push((offset, held_for));
                }
            }
        }
    }
    key_presses
}

fn configure_watching(chip: &mut Chip, offsets: &[u32])
    -> Result<(Vec<LineEventHandle>, Vec<PollFd>), Error>{
    // maps to the driver for the SoC (builtin) GPIO controller.
    let evt_handles = offsets.iter()
        .map(|off| chip.get_line(*off).unwrap())
        .map(|line| line.events(
            LineRequestFlags::INPUT,
            EventRequestFlags::BOTH_EDGES,
            "homeautomation", ).unwrap())
        .collect::<Vec<_>>();

    let pollables = evt_handles.iter()
        .map(|h| PollFd::new(h.as_raw_fd(), 
            PollFlags::POLLIN | 
            PollFlags::POLLPRI))
        .collect::<Vec<_>>();
        
    Ok((evt_handles, pollables))
}

fn to_command(offset: usize, duration: u64) -> Option<Event> {
    if duration > MAX_TAP_LEN {
        match offset {
            27 => Some(Event::Command(Command::MpdNextSong)), //left 3, left
            18 => Some(Event::Command(Command::MpdPrevSong)), //left 3, right
            _ => None,
        }
    } else {
        match offset {
            16 => Some(Event::Command(Command::LampsDim)),
            12 => Some(Event::Command(Command::LampsDimmest)),
            13 => Some(Event::Command(Command::LampsToggle)),
            
            //buttons on desk
            27 => Some(Event::Command(Command::MpdIncreaseVolume)), //left 3, left
            22 => Some(Event::Command(Command::MpdPause)), //left 3, middle
            18 => Some(Event::Command(Command::MpdDecreaseVolume)), //left 3, right
            
            23 => Some(Event::Command(Command::LampsNight)), //right 4, left most
            24 => Some(Event::Command(Command::LampsEvening)), //right 4, left 
            26 => Some(Event::Command(Command::LampsDay)), //right 4, right
            17 => Some(Event::Command(Command::LampsToggle)), //right 4, right most
            _  => None,
        }
    }
}

pub fn start_monitoring(tx: crossbeam_channel::Sender<Event>) 
    -> Result<(), Error> {
      
    if let Some(mut chip) = gpio_cdev::chips()?
        .filter_map(Result::ok)
        .filter(|c| c.label() == "pinctrl-bcm2835")
        .next() {
        
        detect_and_handle(&mut chip, tx)?;
        Ok(())
    } else {
        error!("could not find gpio chip");
        Err(Error::GPIONotFound)
    }
}
