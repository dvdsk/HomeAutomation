use super::{System, Modifications};

pub mod normal;
pub mod lightloop;

trait RoomState {
    fn update(&mut self, mods: &Modifications, system: &mut System);
    fn enter(mods: &mut Modifications, system: &mut System) -> Self;
}