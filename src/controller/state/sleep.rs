use super::super::{Environment, Modifications, System};
use super::{RoomState, State};
use crate::controller::system::mpd_control as mpd;
use crate::errors::Error;

use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct Sleep {
    start: Instant,
}

impl Sleep {
    pub fn setup(mods: &mut Modifications, sys: &mut System) -> Result<Box<dyn RoomState>, Error> {
        dbg!("entering sleep mode");
        mods.reset();

        sys.update_period = Duration::from_secs(60);
        sys.next_update = Instant::now() + sys.update_period;

        const RED: (f32, f32, f32) = (1f32, 0f32, 0f32);
        sys.lights.set_all_rgb(0, RED)?;
        sys.lights.all_on()?;

        Ok(Box::new(Sleep {
            start: Instant::now(),
        }))
    }
}

impl RoomState for Sleep {
    fn update(
        &mut self,
        mods: &mut Modifications,
        sys: &mut System,
        _env: &mut Environment,
    ) -> Result<Option<State>, Error> {
        const LIGHTS_OFF: Duration = Duration::from_secs(60);
        const MUSIC_OFF: Duration = Duration::from_secs(120);

        if self.start.elapsed() >= LIGHTS_OFF && !mods.lighting {
            sys.lights.all_off()?;
        }
        if self.start.elapsed() >= MUSIC_OFF && !mods.mpd {
            mpd::pause()?;
        }

        Ok(None)
    }

    fn breakdown(&self, _: &mut Modifications, _: &mut System) -> Result<(), Error> {
        Ok(())
    }
    fn state(&self) -> State {
        State::LightLoop
    }
}
