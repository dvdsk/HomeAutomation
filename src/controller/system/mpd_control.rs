use crate::errors::Error;
use chrono::Duration;
use rand::Rng;

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

pub fn save_current_playlist(mpd: &mut mpd::Client) -> Result<(),Error>{
    dbg!();
    mpd.pl_remove("temp")?;
    dbg!();
    mpd.save("temp")?;
    dbg!();
    Ok(())
}