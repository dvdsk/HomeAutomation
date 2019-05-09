use super::{State, Modifications, System};
use crate::controller::states;
use crate::controller::states::RoomState;

#[derive(Copy, Clone)]
pub enum TargetState {
    Normal,
    LightLoop,
}

#[derive(Copy, Clone)]
pub enum Command {
  LampsToggle,
  LampsDim,
  LampsDimmest,
  LampsEvening,
  LampsNight,
  LampsDay,
  LampsOff,
  LampsOn,

  ChangeState(TargetState),
}

pub fn handle_cmd(cmd: Command, state: State, mods: &mut Modifications, sys: &mut System) -> State {
    println!("handled a command");
    match cmd {
      Command::ChangeState(target_state) => {
        return handle_changestate_cmd(target_state, mods, sys)
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

fn handle_changestate_cmd(target_state: TargetState, mods: &mut Modifications, sys: &mut System) -> State {
  match target_state {
      TargetState::Normal => State::Normal(states::normal::Normal::enter(mods, sys)),
      TargetState::LightLoop => State::LightLoop(states::lightloop::LightLoop::enter(mods, sys)),
  }
}
