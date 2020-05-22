use super::super::{Modifications, System, Environment};
use super::{RoomState, State};
use crate::controller::system::mpd_control as mpd;

use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct Silent {
}

impl Silent {
    pub fn setup(mods: &mut Modifications, sys: &mut System) -> Self {
        dbg!("entering silent mode");
        mods.reset();
        
        sys.update_period = Duration::from_secs(60);
        sys.next_update = Instant::now()+sys.update_period;

        mpd::pause();
        sys.lights.set_all_ct(50,500);
        sys.lights.single_off(1);

        Silent { }
    }
}

impl RoomState for Silent {
    fn update(&mut self, _: &mut Modifications, _: &mut System, _env: &mut Environment)
         -> Option<State> {
        

        None
    }

    fn breakdown(&self, _: &mut Modifications, _: &mut System) {}
    fn state(&self) -> State {State::LightLoop }
}