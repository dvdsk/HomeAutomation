use super::super::{Environment, Modifications, System};
use super::{RoomState, State};
use crate::errors::Error;

use chrono::{Local, NaiveTime};
use std::time::{Duration, Instant};

fn update_lights(sys: &mut System) {
    fn hour(h: u8) -> NaiveTime {
        NaiveTime::from_hms_opt(h as u32, 0, 0).unwrap()
    }

    let now = Local::now();
    if now.time() > hour(22) || now.time() < hour(6) {
        let _ignore_err = sys.lights.set_all_ct(220, 500);
    } else if now.time() > hour(17) {
        let _ignore_err = sys.lights.set_all_ct(254, 320);
    } else if now.time() >= hour(6) {
        let _ignore_err = sys.lights.set_all_ct(254, 240);
    };
}

#[derive(Default, Clone, Copy)]
pub struct Normal {}

impl Normal {
    pub fn setup(mods: &mut Modifications, sys: &mut System) -> Result<Box<dyn RoomState>, Error> {
        mods.reset();

        sys.update_period = Duration::from_secs(5);
        sys.next_update = Instant::now() + sys.update_period;
        update_lights(sys);
        let _ignore_err = sys.lights.all_on();

        Ok(Box::new(Self::default()))
    }
}

impl RoomState for Normal {
    fn update(
        &mut self,
        mods: &mut Modifications,
        sys: &mut System,
        _env: &mut Environment,
    ) -> Result<Option<State>, Error> {
        //dbg!("updating normal state");
        if !mods.lighting {
            update_lights(sys)
        }
        Ok(None)
    }

    fn breakdown(&self, _: &mut Modifications, _: &mut System) -> Result<(), Error> {
        Ok(())
    }
    fn state(&self) -> State {
        State::Normal
    }
}
