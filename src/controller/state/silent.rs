use super::super::{Environment, Modifications, System};
use super::{RoomState, State};
use crate::controller::system::mpd_control as mpd;
use crate::errors::Error;

use philipshue::hue::LightCommand;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct Silent {}

impl Silent {
    pub fn setup(mods: &mut Modifications, sys: &mut System) -> Result<Box<dyn RoomState>, Error> {
        dbg!("entering silent mode");
        mods.reset();

        sys.update_period = Duration::from_secs(60);
        sys.next_update = Instant::now() + sys.update_period;

        mpd::pause()?;

        sys.lights.single_off(2)?;
        let command = LightCommand::default().on().with_bri(1).with_ct(500);

        for lamp_id in &[1, 8, 5, 4, 6] {
            sys.lights.bridge.set_light_state(*lamp_id, &command)?;
            sys.lights.lamps.get_mut(lamp_id).unwrap().on = true;
        }

        Ok(Box::new(Silent {}))
    }
}

impl RoomState for Silent {
    fn update(
        &mut self,
        _: &mut Modifications,
        _: &mut System,
        _env: &mut Environment,
    ) -> Result<Option<State>, Error> {
        Ok(None)
    }

    fn breakdown(&self, _: &mut Modifications, _: &mut System) -> Result<(), Error> {
        Ok(())
    }
    fn state(&self) -> State {
        State::Silent
    }
}
