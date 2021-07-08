use super::super::{Environment, Modifications, System};
use super::{RoomState, State};
use crate::controller::system;
use retry::{delay::Fixed, retry};

use crate::errors::Error;
use std::time::{Duration, Instant};

const UPDATE_PERIOD: u64 = 5;
const WAKEUP_DURATION: u64 = 60 * 15; //in seconds

const CT_BEGIN: u16 = 500;
const CT_END: u16 = 280;
const CT_PER_SECOND: f32 = ((CT_BEGIN - CT_END) as f32) / (WAKEUP_DURATION as f32);
const BRI_PER_SECOND: f32 = 254. / (WAKEUP_DURATION as f32); //in seconds

const MUSIC_ON: u64 = 60 * 5;
const MAX_VOLUME: f32 = 20.;
const MIN_VOLUME: f32 = 3.;
const VOL_PER_SECOND: f32 = (MAX_VOLUME - MIN_VOLUME) / (WAKEUP_DURATION - MUSIC_ON) as f32;

#[derive(thiserror::Error, Debug)]
pub enum WakeUpStateError {
    #[error("could not setup playlist even after retries: {0}")]
    SetupPlaylist(retry::Error<mpd::error::Error>),

}

#[derive(Clone, Copy)]
pub struct WakeUp {
    start: Instant,
    playing: bool,
}

impl WakeUp {
    fn setup_playlist() -> Result<(), Error> {
        let mpd = &mut retry(Fixed::from_millis(100), || {
            mpd::Client::connect("127.0.0.1:6600")
        }).map_err(WakeUpStateError::SetupPlaylist)?;

        retry(Fixed::from_millis(100).take(3), || {
            system::mpd_control::save_current_playlist(mpd)
        }).map_err(WakeUpStateError::SetupPlaylist)?;

        // note if playlist does not exist we do not 
        // report an error
        retry(Fixed::from_millis(100).take(3), || {
            system::mpd_control::add_from_playlist(
                mpd,
                "calm",
                chrono::Duration::seconds(3 * 60),
                chrono::Duration::seconds(5 * 60),
            )
        }).map_err(WakeUpStateError::SetupPlaylist)?;

        retry(Fixed::from_millis(100).take(3), || {
            system::mpd_control::add_from_playlist(
                mpd,
                "energetic",
                chrono::Duration::seconds(10 * 60),
                chrono::Duration::seconds(11 * 60),
            )
        }).map_err(WakeUpStateError::SetupPlaylist)?;

        retry(Fixed::from_millis(100).take(3), || {
            system::mpd_control::add_from_playlist(
                mpd,
                "active",
                chrono::Duration::seconds(30 * 60),
                chrono::Duration::seconds(60 * 60),
            )
        }).map_err(WakeUpStateError::SetupPlaylist)?;
        Ok(())
    }
}

impl WakeUp {
    pub fn setup(mods: &mut Modifications, sys: &mut System) -> Result<Box<dyn RoomState>, Error> {
        log::info!("starting wakeup state");
        sys.update_period = Duration::from_secs(UPDATE_PERIOD);
        sys.next_update = Instant::now() + sys.update_period;

        // reset modifications on entering state
        mods.lighting = false;
        mods.mpd = false;

        if let Err(e) = Self::setup_playlist() {
            log::error!("could not set up playlist: {}", e);
        }
        sys.lights.set_all_on_ct(0, CT_BEGIN)?;

        Ok(Box::new(Self {
            start: Instant::now(),
            playing: false,
        }))
    }
}

impl RoomState for WakeUp {
    fn update(
        &mut self,
        mods: &mut Modifications,
        sys: &mut System,
        _env: &mut Environment,
    ) -> Result<Option<State>, Error> {
        let elapsed = self.start.elapsed().as_secs();

        if elapsed > WAKEUP_DURATION {
            return Ok(Some(State::Normal));
        }

        // do nothing to the lighting if the user changed it
        if !mods.lighting {
            let bri = (BRI_PER_SECOND * (elapsed as f32)) as u8;
            let ct = CT_BEGIN - (CT_PER_SECOND * (elapsed as f32)) as u16;
            sys.lights.set_all_on_ct(bri, ct)?;
        }

        // do nothing to mpd if the user changed an mpd setting
        if !mods.mpd {
            if !self.playing && elapsed > MUSIC_ON {
                let mut client = mpd::Client::connect("127.0.0.1:6600")?;
                client.volume(MIN_VOLUME as i8)?;
                client.play()?;
                self.playing = true;
            } else if elapsed > MUSIC_ON {
                let since_music_on = elapsed.saturating_sub(MUSIC_ON);
                let mut client = mpd::Client::connect("127.0.0.1:6600")?;
                let volume = MIN_VOLUME + VOL_PER_SECOND * (since_music_on as f32); 
                client.volume(volume as i8)?;
            }
        }

        Ok(None)
    }

    fn breakdown(&self, _: &mut Modifications, _: &mut System) -> Result<(), Error> {
        Ok(())
    }
    fn state(&self) -> State {
        State::WakeUp
    }
}
