use super::super::{Modifications, System, Environment};
use super::{RoomState, ActiveState};

use chrono::{Local, NaiveTime};
use std::time::{Duration, Instant};
use log::error;

fn update_lights(sys: &mut System) {
    let now = Local::now();
    let res = if now.time() > NaiveTime::from_hms(22,0,0) {
        sys.lights.set_all_ct(220,500)
    } else if now.time() > NaiveTime::from_hms(17,0,0) {
        sys.lights.set_all_ct(254,320)
    } else if now.time() > NaiveTime::from_hms(6,0,0) {
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

impl RoomState for Normal {
    fn update(self, _mods: &mut Modifications, sys: &mut System, _env: &mut Environment)
         -> ActiveState {
        
        //dbg!("updating normal state");
        update_lights(sys);
        ActiveState::Normal(self)
    }

    fn enter(_mods: &mut Modifications, sys: &mut System) -> Self {
        dbg!("making everything rdy for the normal state");
        sys.update_period = Duration::from_secs(5);
        sys.next_update = Instant::now()+sys.update_period;

        Self::default()
    }
}