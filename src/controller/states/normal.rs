use super::super::{Modifications, System};
use super::{RoomState};

#[derive(Default)]
pub struct Normal {}

impl RoomState for Normal {
    fn update(&mut self, mods: &Modifications, system: &mut System){
        //dbg!("updating normal state");
    }
    fn enter(mods: &mut Modifications, system: &mut System) -> Self {
        dbg!("making everything rdy for the normal state");
        Self::default()
    }
}