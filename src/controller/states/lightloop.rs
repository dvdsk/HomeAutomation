use super::super::{Modifications, System};
use super::{RoomState};

use std::time::{Duration, Instant};

#[derive(Default)]
pub struct LightLoop {}

impl RoomState for LightLoop {
    fn update(&mut self, _mods: &Modifications, sys: &mut System){
        dbg!("updating lightloop state");
        sys.lights.set_all_to(100, 100);
    }

    fn enter(mods: &mut Modifications, sys: &mut System) -> Self {
        dbg!("making everything rdy for the lightloop state");

        sys.lights.set_all_to(100, 100);
        mods.lighting = false;

        sys.update_period = Duration::from_millis(100);
        sys.next_update = Instant::now()+sys.update_period;

        Self::default()
    }
}



