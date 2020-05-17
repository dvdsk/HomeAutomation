use serde::{Serialize, Deserialize};
use super::{System, Modifications, Environment};

mod normal;
mod lightloop;
mod wakeup;

pub use normal::Normal;
pub use lightloop::LightLoop;
pub use wakeup::WakeUp;

#[derive(Copy, Clone)]
pub enum ActiveState {
    Normal(normal::Normal),
    LightLoop(lightloop::LightLoop),
    WakeUp(wakeup::WakeUp),
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum TargetState {
    Normal,
    LightLoop,
    WakeUp,
}

impl ActiveState {
    pub fn update(self, mods: &mut Modifications, system: &mut System, env: &mut Environment) -> ActiveState {
        match self {
            ActiveState::Normal(state) => state.update(mods, system, env),
            ActiveState::LightLoop(state) => state.update(mods, system, env),           
            ActiveState::WakeUp(state) => state.update(mods, system, env),   
        }
    }
}

pub fn change_state(target_state: TargetState, mods: &mut Modifications, sys: &mut System) -> ActiveState {
    match target_state {
        TargetState::Normal => ActiveState::Normal(Normal::enter(mods, sys)),
        TargetState::LightLoop => ActiveState::LightLoop(LightLoop::enter(mods, sys)),
        TargetState::WakeUp => ActiveState::WakeUp(WakeUp::enter(mods, sys)),
    }
}

pub trait RoomState {
    fn enter(mods: &mut Modifications, system: &mut System) -> Self;
    fn update(self, mods: &mut Modifications, system: &mut System, env: &mut Environment) -> ActiveState;
}