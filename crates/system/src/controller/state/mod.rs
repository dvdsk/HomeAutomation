use super::{Environment, Modifications, System};
use crate::errors::Error;
use serde::{Deserialize, Serialize};

mod lightloop;
mod normal;
mod off;
mod silent;
mod sleep;
mod wakeup;

pub use lightloop::LightLoop;
pub use normal::Normal;
pub use off::Off;
pub use silent::Silent;
pub use sleep::Sleep;
pub use wakeup::{WakeUp, WakeUpStateError};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum State {
    Normal,
    LightLoop,
    WakeUp,
    Sleep,
    Silent,
    Off,
}

pub trait RoomState {
    fn update(
        &mut self,
        mods: &mut Modifications,
        system: &mut System,
        env: &mut Environment,
    ) -> Result<Option<State>, Error>;
    fn breakdown(&self, mods: &mut Modifications, system: &mut System) -> Result<(), Error>;
    fn state(&self) -> State;
}
