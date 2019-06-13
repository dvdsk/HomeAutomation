use super::super::{Modifications, System};
use super::{RoomState, ActiveState};
use super::Normal;

use std::time::{Duration, Instant};

const WAKEUP_DURATION: u64 = 900; //in seconds
const BRI_PER_SECOND: f32 = 254./(WAKEUP_DURATION as f32); //in seconds

const CT_BEGIN: u16 = 500; const CT_END: u16 = 280;
const CT_PER_SECOND: f32 = ((CT_BEGIN-CT_END) as f32)/(WAKEUP_DURATION as f32);
const UPDATE_PERIOD: u64 = 5;

#[derive(Clone, Copy)]
pub struct WakeUp {
    start: Instant,
}

impl RoomState for WakeUp {
    fn update(self, mods: &mut Modifications, sys: &mut System) -> ActiveState {
        //dbg!("updating normal state");
        let elapsed = self.start.elapsed().as_secs();
        
        if elapsed > WAKEUP_DURATION {
            sys.lights.set_all_to(254, CT_END).unwrap();
            ActiveState::Normal(Normal::enter(mods, sys))
        } else {
            let bri = (BRI_PER_SECOND*(elapsed as f32)) as u8;
            let ct = CT_BEGIN-(CT_PER_SECOND*(elapsed as f32)) as u16;

            sys.lights.set_all_to(bri, ct).unwrap();
            
            ActiveState::WakeUp(self)
        }
    }

    fn enter(_mods: &mut Modifications, sys: &mut System) -> Self {
        dbg!("starting wakeup state");
        sys.update_period = Duration::from_secs(UPDATE_PERIOD);
        sys.next_update = Instant::now()+sys.update_period;

        sys.lights.set_all_to(0, CT_BEGIN).unwrap();
        Self{start: Instant::now()}
    }
}
