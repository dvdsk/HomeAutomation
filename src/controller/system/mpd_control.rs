use crate::errors::Error;
use chrono::Duration;
use rand::Rng;

use crate::input::mpd_status::MpdStatus;

pub fn add_from_playlist(mpd: &mut mpd::Client, name: &str, minimal_play_time: Duration, maximal_play_time: Duration) -> Result<(),Error>{
    let mut rng = rand::thread_rng();

    let songs = mpd.playlist(name);
    let mut songs = match songs{ //report all errors except non existing playlist
        Ok(songs) => songs,
        Err(error) => match error {
            mpd::error::Error::Server(serv_error) => match serv_error.code {
                mpd::error::ErrorCode::NoExist => {warn!("could not find playlist: {}", name); return Ok(())},
                _ => return Err(mpd::error::Error::Server(serv_error).into()),
            },
            _ => return Err(error.into()),
        },
    };

    let mut time = Duration::seconds(0);
    //add random songs until the playtime is larger then the minimum
    while time < minimal_play_time && songs.len()!=0 {
        let idx = rng.gen_range(0, songs.len()-1);
        let song = songs.remove(idx);
        if time + song.duration.unwrap() < maximal_play_time {
            time = time + song.duration.unwrap();
            mpd.push(song)?;
        }
        dbg!(songs.len());
    }
    Ok(())
}

pub fn increase_volume(mpd_status: &mut MpdStatus) -> Result<(),Error>{
    const VOLUME_INCREMENT: i8 = 1;

    let mut client = mpd::Client::connect("127.0.0.1:6600")?;
    let current_volume = mpd_status.get_volume();
    if current_volume + VOLUME_INCREMENT > 100 {return Ok(()); }

    client.volume(current_volume+VOLUME_INCREMENT)?;
    Ok(())
}

pub fn decrease_volume(mpd_status: &mut MpdStatus) -> Result<(),Error>{
    const VOLUME_INCREMENT: i8 = 1;

    let mut client = mpd::Client::connect("127.0.0.1:6600")?;
    let current_volume = mpd_status.get_volume();
    if current_volume - VOLUME_INCREMENT < 0 {return Ok(()); }

    client.volume(current_volume-VOLUME_INCREMENT)?;
    Ok(())
}

pub fn toggle_playback(mpd_status: &mut MpdStatus) -> Result<(), Error>{
    let mut client = mpd::Client::connect("127.0.0.1:6600")?;
    
    match mpd_status.is_playing() {
        mpd::status::State::Stop => client.play()?,
        mpd::status::State::Pause => client.toggle_pause()?,
        mpd::status::State::Play => client.toggle_pause()?,
    }
    Ok(())
}

pub fn save_current_playlist(mpd: &mut mpd::Client) -> Result<(),Error>{
    dbg!();
    mpd.pl_remove("temp")?;
    dbg!();
    mpd.save("temp")?;
    dbg!();
    Ok(())
}