use super::super::{Modifications, System, Environment};
use super::{RoomState, ActiveState};

use std::time::{Duration, Instant};

#[derive(Default, Clone, Copy)]
pub struct Normal {}

impl RoomState for Normal {
    fn update(self, _mods: &mut Modifications, _sys: &mut System, _env: &mut Environment) -> ActiveState {
        //dbg!("updating normal state");
        ActiveState::Normal(self)
    }
    fn enter(_mods: &mut Modifications, sys: &mut System) -> Self {
        dbg!("making everything rdy for the normal state");
        sys.update_period = Duration::from_secs(5);
        sys.next_update = Instant::now()+sys.update_period;

        Self::default()
    }
}
