use super::super::{Modifications, System, Environment};
use super::{RoomState, State};
use crate::controller::system::mpd_control as mpd;
use crate::errors::Error;

use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct Quiet {
}

impl Quiet {
    pub fn setup(mods: &mut Modifications, sys: &mut System) -> Result<Box<dyn RoomState>, Error> {
        dbg!("entering silent mode");
        mods.reset();
        
        sys.update_period = Duration::from_secs(60);
        sys.next_update = Instant::now()+sys.update_period;

        mpd::set_volume(10)?;
        
        sys.lights.set_all_ct(1,500)?;
        sys.lights.single_off(2)?;

        Ok(Box::new(Quiet { }))
    }
}

impl RoomState for Quiet {
    fn update(&mut self, _: &mut Modifications, _: &mut System, _env: &mut Environment)
         -> Result<Option<State>, Error> {
        
        Ok(None)
    }

    fn breakdown(&self, _: &mut Modifications, _sys: &mut System) -> Result<(), Error> {Ok(())}
    fn state(&self) -> State {State::Quite }
}