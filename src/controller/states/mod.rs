use super::{System, Modifications};

mod normal;
mod lightloop;

pub use normal::Normal;
pub use lightloop::LightLoop;


#[derive(Copy, Clone)]
pub enum ActiveState {
    Normal(normal::Normal),
    LightLoop(lightloop::LightLoop)
}

impl ActiveState {
    pub fn update(self, mods: &mut Modifications, system: &mut System) -> ActiveState {
        match self {
            ActiveState::Normal(state) => state.update(mods, system),
            ActiveState::LightLoop(state) => state.update(mods, system),           
        }
    }
}

pub trait RoomState {
    fn enter(mods: &mut Modifications, system: &mut System) -> Self;
    fn update(self, mods: &mut Modifications, system: &mut System) -> ActiveState;
}