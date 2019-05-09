use super::super::{Modifications, System};
use super::{RoomState, State};

#[derive(Default, Clone, Copy)]
pub struct Normal {}

impl RoomState for Normal {
    fn update(self, mods: &Modifications, system: &mut System) -> State {
        //dbg!("updating normal state");
        State::Normal(self)
    }
    fn enter(mods: &mut Modifications, system: &mut System) -> Self {
        dbg!("making everything rdy for the normal state");
        Self::default()
    }
}
