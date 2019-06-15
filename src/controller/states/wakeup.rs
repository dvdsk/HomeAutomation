use super::super::{Modifications, System};
use super::{RoomState, ActiveState};
use super::Normal;
use crate::controller::system;
use crate::errors::Error;
use retry::{retry, delay::Fixed};

use std::time::{Duration, Instant};

const UPDATE_PERIOD: u64 = 5;
const WAKEUP_DURATION: u64 = 50*15; //in seconds

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
        let mpd = &mut mpd::Client::connect("127.0.0.1:6600")?;
        let mpd = &mut retry(Fixed::from_millis(100), || mpd::Client::connect("127.0.0.1:6600"))?;
        retry(Fixed::from_millis(100), || system::mpd_control::save_current_playlist(mpd))?;
        retry(Fixed::from_millis(100), || 
            system::mpd_control::add_from_playlist(mpd, "calm", 
                chrono::Duration::seconds(3*60), 
                chrono::Duration::seconds(5*60))
        )?;
        retry(Fixed::from_millis(100), || 
            system::mpd_control::add_from_playlist(mpd, "energetic", 
                chrono::Duration::seconds(10*60), 
                chrono::Duration::seconds(11*60))
        )?;
        retry(Fixed::from_millis(100), || 
            system::mpd_control::add_from_playlist(mpd, "active", 
                chrono::Duration::seconds(30*60), 
                chrono::Duration::seconds(60*60))   
        )?;
        Ok(())     
    }

}

impl RoomState for WakeUp {
    fn update(self, mods: &mut Modifications, sys: &mut System) -> ActiveState {
        //dbg!("updating normal state");
        let elapsed = self.start.elapsed().as_secs();
        
        if elapsed > WAKEUP_DURATION {
            sys.lights.set_all_to(254, CT_END).unwrap();
            return ActiveState::Normal(Normal::enter(mods, sys))
        }
    
        let bri = (BRI_PER_SECOND*(elapsed as f32)) as u8;
        let ct = CT_BEGIN-(CT_PER_SECOND*(elapsed as f32)) as u16;

        sys.lights.set_all_to(bri, ct).unwrap();
        
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

        ActiveState::WakeUp(self)

    }

    fn enter(_mods: &mut Modifications, sys: &mut System) -> Self {
        dbg!("starting wakeup state");
        sys.update_period = Duration::from_secs(UPDATE_PERIOD);
        sys.next_update = Instant::now()+sys.update_period;

        Self::setup_playlist();
        sys.lights.set_all_to(0, CT_BEGIN);
        Self{start: Instant::now(), playing: false}
    }
}
