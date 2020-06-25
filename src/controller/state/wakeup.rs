use super::super::{Modifications, System, Environment};
use super::{RoomState, State};
use crate::controller::system;
use crate::errors::Error;
use retry::{retry, delay::Fixed};

use std::time::{Duration, Instant};

const UPDATE_PERIOD: u64 = 5;
const WAKEUP_DURATION: u64 = 60*15; //in seconds

const CT_BEGIN: u16 = 500; const CT_END: u16 = 280;
const CT_PER_SECOND: f32 = ((CT_BEGIN-CT_END) as f32)/(WAKEUP_DURATION as f32);
const BRI_PER_SECOND: f32 = 254./(WAKEUP_DURATION as f32); //in seconds

const MUSIC_ON: u64 = 60*5;
const MAX_VOLUME: i8 = 40;
const MIN_VOLUME: i8 = 10;
const VOL_PER_SECOND: f32 = (MAX_VOLUME-MIN_VOLUME) as f32/(WAKEUP_DURATION-MUSIC_ON) as f32;

#[derive(Clone, Copy)]
pub struct WakeUp {
    start: Instant,
    playing: bool,
}

impl WakeUp {

    fn setup_playlist() -> Result<(), Error> {
        let mpd = &mut retry(Fixed::from_millis(100), || mpd::Client::connect("127.0.0.1:6600"))?;
        dbg!();
        retry(Fixed::from_millis(100).take(3), || system::mpd_control::save_current_playlist(mpd))?;
        dbg!();
        retry(Fixed::from_millis(100).take(3), || 
            system::mpd_control::add_from_playlist(mpd, "calm", 
                chrono::Duration::seconds(3*60), 
                chrono::Duration::seconds(5*60))
        )?;
        dbg!();
        retry(Fixed::from_millis(100).take(3), || 
            system::mpd_control::add_from_playlist(mpd, "energetic", 
                chrono::Duration::seconds(10*60), 
                chrono::Duration::seconds(11*60))
        )?;
        retry(Fixed::from_millis(100).take(3), || 
            system::mpd_control::add_from_playlist(mpd, "active", 
                chrono::Duration::seconds(30*60), 
                chrono::Duration::seconds(60*60))   
        )?;
        Ok(())     
    }

}

impl WakeUp {
    pub fn setup(mods: &mut Modifications, sys: &mut System) -> Result<Box<dyn RoomState>, Error> {
        dbg!("starting wakeup state");
        sys.update_period = Duration::from_secs(UPDATE_PERIOD);
        sys.next_update = Instant::now()+sys.update_period;

        // reset modifications on entering state
        mods.lighting = false;
        mods.mpd = false;

        Self::setup_playlist()?;
        sys.lights.set_all_ct(0, CT_BEGIN)?;
        
        Ok(Box::new(Self{start: Instant::now(), playing: false}))
    }
}

impl RoomState for WakeUp {
    fn update(&mut self, mods: &mut Modifications, sys: &mut System, _env: &mut Environment) -> Result<Option<State>, Error> {
        let elapsed = self.start.elapsed().as_secs();
        
        if elapsed > WAKEUP_DURATION {
            return Ok(Some(State::Normal));
        }
    
        if !mods.lighting { // if lighting controls have not been modified externally since start
            if sys.lights.numb_on() < 3 {
                sys.lights.all_on()?;
            }
            
            let bri = (BRI_PER_SECOND*(elapsed as f32)) as u8;
            let ct = CT_BEGIN-(CT_PER_SECOND*(elapsed as f32)) as u16;
            sys.lights.set_all_ct(bri, ct)?; //TODO map to terror error
        }

        if !mods.mpd { // if mpd controls have not been modified externally since start
            if !self.playing {
                if elapsed > MUSIC_ON {
                    mpd::Client::connect("127.0.0.1:6600")
                        .and_then(|ref mut c| c.volume(MIN_VOLUME)
                        .and_then(|_| c.play())); //only play if the volume was set correctly
                }
            } else {
                mpd::Client::connect("127.0.0.1:6600")
                    .and_then(|mut c| c.volume((VOL_PER_SECOND*(elapsed-MUSIC_ON) as f32) as i8 ));
            }
        }

        Ok(None)
    }

    fn breakdown(&self, _: &mut Modifications, _: &mut System) -> Result<(), Error> {Ok(())}
    fn state(&self) -> State {State::WakeUp }
}
