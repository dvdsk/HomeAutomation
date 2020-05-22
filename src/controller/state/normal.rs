use super::super::{Modifications, System, Environment};
use super::{RoomState, State};

use chrono::{Local, NaiveTime};
use std::time::{Duration, Instant};
use log::error;

fn update_lights(sys: &mut System) {
    let now = Local::now();
    let res = if now.time() > NaiveTime::from_hms(22,0,0) || now.time() < NaiveTime::from_hms(6,0,0) {
        sys.lights.set_all_ct(220,500)
    } else if now.time() > NaiveTime::from_hms(17,0,0) {
        sys.lights.set_all_ct(254,320)
    } else if now.time() >= NaiveTime::from_hms(6,0,0) {
        sys.lights.set_all_ct(254,240)
    } else {
        return;
    };

    if let Err(e) = res {
        error!("could not update lights: {:?}", e);
    }
}


#[derive(Default, Clone, Copy)]
pub struct Normal {}

impl Normal {
    pub fn setup(mods: &mut Modifications, sys: &mut System) -> Self {
        dbg!("making everything rdy for the normal state");
        mods.reset();
        
        sys.update_period = Duration::from_secs(5);
        sys.next_update = Instant::now()+sys.update_period;
        update_lights(sys);

        Self::default()
    }
}

impl RoomState for Normal {
    fn update(&mut self, mods: &mut Modifications, sys: &mut System, _env: &mut Environment) -> Option<State> {
        
        //dbg!("updating normal state");
        if !mods.lighting {update_lights(sys);}
        None
    }

    fn breakdown(&self, _: &mut Modifications, _: &mut System) {}
    fn state(&self) -> State {State::LightLoop }
}