use serde::{Serialize, Deserialize};

use super::{ActiveState, Modifications, System};
use crate::controller::states::*;
use crate::controller::system::mpd_control;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Command {
  None,

  LampsToggle,
  LampsDim,
  LampsDimmest,
  LampsEvening,
  LampsNight,
  LampsDay,

  MpdPause,
  MpdDecreaseVolume,
  MpdIncreaseVolume,
  MpdNextSong,
  MpdPrevSong,

  GoToLinux,
  GoToWindows,
  
  Test, //prints a test message

  ChangeState(TargetState),
}

//TODO handle error on command
pub fn handle_cmd(cmd: Command, state: ActiveState, mods: &mut Modifications, sys: &mut System) -> ActiveState {
    println!("handled a command");
    match cmd {
      Command::ChangeState(target_state) => {
        return change_state(target_state, mods, sys)
      }
      //Command::PauseMpd => {unimplemented!(); state},

        Command::LampsToggle => {sys.lights.toggle().unwrap(); mods.lighting = true}
        Command::LampsDim => {sys.lights.set_all_to(50,500).unwrap(); mods.lighting = true}
        Command::LampsDimmest => {sys.lights.set_all_to(1,500).unwrap(); mods.lighting = true}
        Command::LampsEvening => {sys.lights.set_all_to(254,320).unwrap(); mods.lighting = true}
        Command::LampsNight => {sys.lights.set_all_to(220,500).unwrap(); mods.lighting = true}
        Command::LampsDay => {sys.lights.set_all_to(254,240).unwrap(); mods.lighting = true}

        Command::GoToLinux => println!("should go to linux"),
        Command::GoToWindows => println!("should go to windows"),
        
        Command::MpdPause => {mpd_control::toggle_playback(&mut sys.mpd).unwrap(); mods.mpd = true },
        Command::MpdDecreaseVolume => {mpd_control::increase_volume(&mut sys.mpd).unwrap(); mods.mpd = true },
        Command::MpdIncreaseVolume => {mpd_control::decrease_volume(&mut sys.mpd).unwrap(); mods.mpd = true },
        Command::MpdNextSong => {mpd_control::next_song().unwrap()},
        Command::MpdPrevSong => {mpd_control::prev_song().unwrap()},

        Command::Test => { warn!("The Test command was just send")},
        Command::None => { error!("None command was issued, this should not happen")},
    }
    state
}
