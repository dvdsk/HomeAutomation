use super::super::{Modifications, System};
use super::{RoomState, ActiveState};
use super::normal;

use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy)]
pub struct LightLoop {
    counter: u8,
}

impl RoomState for LightLoop {
    fn update(mut self, mods: &mut Modifications, sys: &mut System) -> ActiveState {
        dbg!("updating lightloop state");
        if self.counter == 0 {
            ActiveState::Normal(normal::Normal::enter(mods, sys))
        } else {
            dbg!(self.counter);
            sys.lights.set_all_to(self.counter, 200).unwrap();
            self.counter -= 1;
            ActiveState::LightLoop(self)
        }
    }
    fn enter(mods: &mut Modifications, sys: &mut System) -> Self {
        dbg!("making everything rdy for the lightloop state");

        sys.lights.set_all_to(0, 200).unwrap();
        mods.lighting = false;

        sys.update_period = Duration::from_millis(500);
        sys.next_update = Instant::now()+sys.update_period;

        Self {counter: 100, }
    }
}



