use super::{System, Modifications};

pub mod normal;
pub mod lightloop;

#[derive(Copy, Clone)]
pub enum State {
    Normal(normal::Normal),
    LightLoop(lightloop::LightLoop)
}

impl State {
    pub fn update(self, mods: &Modifications, system: &mut System) -> State {
        match self {
            State::Normal(state) => state.update(mods, system),
            State::LightLoop(state) => state.update(mods, system),           
        }
    }
}

pub trait RoomState {
    fn enter(mods: &mut Modifications, system: &mut System) -> Self;
    fn update(self, mods: &Modifications, system: &mut System) -> State;
}