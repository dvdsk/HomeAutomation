use serde::{Serialize, Deserialize};
use super::{System, Modifications, Environment};

mod normal;
mod lightloop;
mod wakeup;
mod sleep;
mod silent;

pub use normal::Normal;
pub use lightloop::LightLoop;
pub use wakeup::WakeUp;
pub use sleep::Sleep;
pub use silent::Silent;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum State {
    Normal,
    LightLoop,
    WakeUp,
    Sleep,
    Silent,
}

pub trait RoomState {
    fn update(&mut self, mods: &mut Modifications, system: &mut System, env: &mut Environment) -> Option<State>;
    fn breakdown(&self, mods: &mut Modifications, system: &mut System);
    fn state(&self) -> State;
}