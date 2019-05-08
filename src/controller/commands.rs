use super::{Command, State, Modifications, System};
use crate::controller::states;

pub fn handle_cmd(cmd: Command, state: State, mods: &mut Modifications, sys: &mut System) -> State {
    println!("handled a command");
    match cmd {
      Command::ChangeState(target_state) => {
          handle_changestate_cmd(&target_state, mods, sys);
          return target_state
      }
      //Command::PauseMpd => {unimplemented!(); state},

        Command::LampsToggle => {sys.lights.toggle().unwrap(); mods.lighting = true},
        Command::LampsDim => {sys.lights.set_all_to(50,500).unwrap(); mods.lighting = true},
        Command::LampsDimmest => {sys.lights.set_all_to(1,500).unwrap(); mods.lighting = true},
        Command::LampsEvening => {sys.lights.set_all_to(254,320).unwrap(); mods.lighting = true},
        Command::LampsNight => {sys.lights.set_all_to(220,500).unwrap(); mods.lighting = true},
        Command::LampsDay => {sys.lights.set_all_to(254,240).unwrap(); mods.lighting = true},
        Command::LampsOff => {unimplemented!(); mods.lighting = true},
        Command::LampsOn => {unimplemented!(); mods.lighting = true},
    }
    state
}

fn handle_changestate_cmd(target_state: &State, mods: &mut Modifications, sys: &mut System){
    match target_state {
        State::Normal => states::normal::enter(),
        State::LightLoop => states::lightloop::enter(mods, sys),
        State::Other => println!("setting up other state"),
    }
}
