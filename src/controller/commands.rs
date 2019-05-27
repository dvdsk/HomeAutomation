use super::{ActiveState, Modifications, System};
use crate::controller::states::*;

#[derive(Copy, Clone)]
pub enum Command {
  LampsToggle,
  LampsDim,
  LampsDimmest,
  LampsEvening,
  LampsNight,
  LampsDay,

  PauseMpd,
  GoToLinux,
  GoToWindows,
  
  Test, //prints a test message

  ChangeState(TargetState),
}

pub fn handle_cmd(cmd: Command, state: ActiveState, mods: &mut Modifications, sys: &mut System) -> ActiveState {
    println!("handled a command");
    match cmd {
      Command::ChangeState(target_state) => {
        return change_state(target_state, mods, sys)
      }
      //Command::PauseMpd => {unimplemented!(); state},

        Command::LampsToggle => {sys.lights.toggle().unwrap(); mods.lighting = true},
        Command::LampsDim => {sys.lights.set_all_to(50,500).unwrap(); mods.lighting = true},
        Command::LampsDimmest => {sys.lights.set_all_to(1,500).unwrap(); mods.lighting = true},
        Command::LampsEvening => {sys.lights.set_all_to(254,320).unwrap(); mods.lighting = true},
        Command::LampsNight => {sys.lights.set_all_to(220,500).unwrap(); mods.lighting = true},
        Command::LampsDay => {sys.lights.set_all_to(254,240).unwrap(); mods.lighting = true},

        Command::GoToLinux => println!("should go to linux"),
        Command::GoToWindows => println!("should go to windows"),
        
        Command::PauseMpd => println!("should pause mpd"),

        Command::Test => { warn!("The Test command was just send")},
    }
    state
}
