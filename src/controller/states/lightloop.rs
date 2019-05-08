use super::super::{Modifications, System};
use std::time::{Duration, Instant};

pub fn update(_mods: &Modifications, _system: &System){
    dbg!("updating lightloop state");
    sys.lights.set_all_to(100, 100);
}

pub fn enter(mods: &mut Modifications, sys: &mut System){
    dbg!("making everything rdy for the lightloop state");

    sys.lights.set_all_to(100, 100);
    mods.lighting = false;

    sys.update_period = Duration::from_millis(100);
    sys.next_update = Instant::now()+sys.update_period;
}
