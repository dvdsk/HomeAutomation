use super::super::{Modifications, System, Environment};
use super::{RoomState, State};
use crate::controller::system::mpd_control as mpd;

use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub struct Sleep {
    start: Instant,
}

impl Sleep {
    pub fn setup(mods: &mut Modifications, sys: &mut System) -> Self {
        dbg!("entering sleep mode");
        mods.reset();
        
        sys.update_period = Duration::from_secs(60);
        sys.next_update = Instant::now()+sys.update_period;

        const RED: (f32,f32,f32) = (1f32, 0f32, 0f32);
        sys.lights.set_all_rgb(0, RED);

        Sleep {start: Instant::now()}
    }
}

impl RoomState for Sleep {

    fn update(&mut self, mods: &mut Modifications, sys: &mut System, _env: &mut Environment) -> Option<State> {
        const LIGHTS_OFF: Duration = Duration::from_secs(60);
        const MUSIC_OFF: Duration = Duration::from_secs(120);

        if self.start.elapsed() >= LIGHTS_OFF && !mods.lighting {
            sys.lights.off();
        }
        if self.start.elapsed() >= MUSIC_OFF && !mods.mpd {
            mpd::pause();
        }
        
        None
    }

    fn breakdown(&self, _: &mut Modifications, _: &mut System) {}
    fn state(&self) -> State {State::LightLoop }
}