use serde::{Serialize, Deserialize};

use super::{Modifications, System};
use crate::controller::state::*;
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

  ChangeState(State),
}

//TODO handle error on command
pub fn handle_cmd(cmd: Command, mods: &mut Modifications, sys: &mut System) -> Option<State> {

  println!("handled a command");
    match cmd {
      Command::ChangeState(target_state) => {
        return Some(target_state);
      }
      //Command::PauseMpd => {unimplemented!(); state},

        Command::LampsToggle => {sys.lights.toggle().unwrap(); mods.lighting = true}
        Command::LampsDim => {sys.lights.set_all_ct(50,500).unwrap(); mods.lighting = true}
        Command::LampsDimmest => {sys.lights.set_all_ct(1,500).unwrap(); mods.lighting = true}
        Command::LampsEvening => {sys.lights.set_all_ct(254,320).unwrap(); mods.lighting = true}
        Command::LampsNight => {sys.lights.set_all_ct(220,500).unwrap(); mods.lighting = true}
        Command::LampsDay => {sys.lights.set_all_ct(254,240).unwrap(); mods.lighting = true}

        Command::GoToLinux => println!("should go to linux"),
        Command::GoToWindows => println!("should go to windows"),
        
        Command::MpdPause => {mpd_control::toggle_playback(&mut sys.mpd).unwrap(); mods.mpd = true },
        Command::MpdDecreaseVolume => {mpd_control::decrease_volume(&mut sys.mpd).unwrap(); mods.mpd = true },
        Command::MpdIncreaseVolume => {
            if let mpd::State::Play = sys.mpd.is_playing() {
                mpd_control::increase_volume(&mut sys.mpd).unwrap(); 
                mods.mpd = true 
            }
        },
        Command::MpdNextSong => {mpd_control::next_song().unwrap()},
        Command::MpdPrevSong => {mpd_control::prev_song().unwrap()},

        Command::Test => { warn!("The Test command was just send")},
        Command::None => { error!("None command was issued, this should not happen")},
    }
    None
}
