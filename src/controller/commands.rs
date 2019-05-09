use super::{ActiveState, Modifications, System};
use crate::controller::states::*;
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

pub fn handle_cmd(cmd: Command, state: ActiveState, mods: &mut Modifications, sys: &mut System) -> ActiveState {
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

fn handle_changestate_cmd(target_state: TargetState, mods: &mut Modifications, sys: &mut System) -> ActiveState {
  match target_state {
      TargetState::Normal => ActiveState::Normal(Normal::enter(mods, sys)),
      TargetState::LightLoop => ActiveState::LightLoop(LightLoop::enter(mods, sys)),
  }
}
