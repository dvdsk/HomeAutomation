use serde::{Serialize, Deserialize};
use super::{System, Modifications, Environment};
use crate::errors::Error;

mod normal;
mod lightloop;
mod wakeup;
mod sleep;
mod silent;
mod quiet;
mod off;

pub use normal::Normal;
pub use lightloop::LightLoop;
pub use wakeup::WakeUp;
pub use sleep::Sleep;
pub use silent::Silent;
pub use quiet::Quiet;
pub use off::Off;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum State {
    Normal,
    LightLoop,
    WakeUp,
    Sleep,
    Silent,
    Quiet,
    Off,
}

pub trait RoomState {
    fn update(&mut self, mods: &mut Modifications, system: &mut System, 
        env: &mut Environment) -> Result<Option<State>, Error>;
    fn breakdown(&self, mods: &mut Modifications, system: &mut System)
         -> Result<(), Error>;
    fn state(&self) -> State;
}