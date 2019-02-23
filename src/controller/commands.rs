use super::{Command, State, Modification, System};
use crate::controller::states;

pub fn handle_cmd(cmd: Command, state: State, mods: &mut Vec<Modification>, sys: &mut System) -> State {
    println!("handled a command");
    match cmd {
      Command::ChangeState(target_state) => {
          handle_changestate_cmd(&target_state);
          mods.clear();
          return target_state
      }
      //Command::PauseMpd => {unimplemented!(); state},

      Command::LampsToggle => {sys.lights.toggle(); mods.push(Modification::Lighting)},
			Command::LampsDim => {sys.lights.set_all_to(50,500); mods.push(Modification::Lighting)},
			Command::LampsDimmest => {sys.lights.set_all_to(1,500); mods.push(Modification::Lighting)},
			Command::LampsEvening => {sys.lights.set_all_to(254,320); mods.push(Modification::Lighting)},
			Command::LampsNight => {sys.lights.set_all_to(220,500); mods.push(Modification::Lighting)},
			Command::LampsDay => {sys.lights.set_all_to(254,240); mods.push(Modification::Lighting)},
			Command::LampsOff => {unimplemented!(); mods.push(Modification::Lighting)},
			Command::LampsOn => {unimplemented!(); mods.push(Modification::Lighting)},
    }
    state
}

fn handle_changestate_cmd(target_state: &State){
    match target_state {
        State::Normal => states::normal::enter(),
        State::Other => println!("setting up other state"),
    }
}
