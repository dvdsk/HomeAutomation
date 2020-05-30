use super::super::{Modifications, System, Environment};
use super::{RoomState, State};
use crate::errors::Error;
use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy)]
pub struct LightLoop {
    counter: u8,
}

impl LightLoop {
    pub fn setup(mods: &mut Modifications, sys: &mut System) -> Result<Box<dyn RoomState>, Error> {
        dbg!("making everything rdy for the lightloop state");

        sys.lights.set_all_ct(0, 200)?;
        mods.lighting = false;

        sys.update_period = Duration::from_millis(500);
        sys.next_update = Instant::now()+sys.update_period;

        Ok(Box::new(Self {counter: 100, }))
    }
}

impl RoomState for LightLoop {
    fn update(&mut self, mods: &mut Modifications, sys: &mut System, _env: &mut Environment) -> Result<Option<State>, Error> {
        dbg!("updating lightloop state");
        if self.counter == 0 {
            Ok(Some(State::Normal))
        } else {
            if !mods.lighting {
                dbg!(self.counter);
                sys.lights.set_all_ct(self.counter, 200)?;
                self.counter -= 1;
            }
            Ok(None)
        }
    }
    fn breakdown(&self, _: &mut Modifications, _: &mut System) -> Result<(), Error> {Ok(())}
    fn state(&self) -> State {State::LightLoop }
}



