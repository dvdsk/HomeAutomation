use super::super::{Modifications, System, Environment};
use super::{RoomState, State};

use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy)]
pub struct LightLoop {
    counter: u8,
}

impl LightLoop {
    pub fn setup(mods: &mut Modifications, sys: &mut System) -> Self {
        dbg!("making everything rdy for the lightloop state");

        sys.lights.set_all_ct(0, 200).unwrap();
        mods.lighting = false;

        sys.update_period = Duration::from_millis(500);
        sys.next_update = Instant::now()+sys.update_period;

        Self {counter: 100, }
    }
}

impl RoomState for LightLoop {
    fn update(&mut self, mods: &mut Modifications, sys: &mut System, _env: &mut Environment) -> Option<State> {
        dbg!("updating lightloop state");
        if self.counter == 0 {
            Some(State::Normal)
        } else {
            dbg!(self.counter);
            sys.lights.set_all_ct(self.counter, 200).unwrap();
            self.counter -= 1;
            None
        }
    }
    fn breakdown(&self, _: &mut Modifications, _: &mut System) {}
    fn state(&self) -> State {State::LightLoop }
}



