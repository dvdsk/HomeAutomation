use super::super::{Environment, Modifications, System};
use super::{RoomState, State};
use crate::controller::system::mpd_control as mpd;
use crate::errors::Error;

#[derive(Clone, Copy)]
pub struct Off {}

impl Off {
    pub fn setup(mods: &mut Modifications, sys: &mut System) -> Result<Box<dyn RoomState>, Error> {
        mods.reset();
        mpd::pause()?;
        sys.lights.all_off()?;
        Ok(Box::new(Off {}))
    }
}

impl RoomState for Off {
    fn update(
        &mut self,
        _: &mut Modifications,
        _: &mut System,
        _env: &mut Environment,
    ) -> Result<Option<State>, Error> {
        Ok(None)
    }

    fn breakdown(&self, _: &mut Modifications, _sys: &mut System) -> Result<(), Error> {
        Ok(())
    }
    fn state(&self) -> State {
        State::Off
    }
}
