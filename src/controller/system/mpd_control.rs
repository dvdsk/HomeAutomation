use crate::errors::Error;
use chrono::Duration;
use rand::Rng;

pub fn add_from_playlist(mpd: &mut mpd::Client, name: &str, minimal_play_time: Duration, maximal_play_time: Duration) -> Result<(),Error>{
    let mut rng = rand::thread_rng();

    let mut songs = mpd.playlist(name)?;
    let mut time = Duration::seconds(0);
    //add random songs until the playtime is larger then the minimum
    while time < minimal_play_time && songs.len()!=0 {
        let idx = rng.gen_range(0, songs.len()-1);
        let song = songs.remove(idx);
        if time + song.duration.unwrap() < maximal_play_time {
            time = time + song.duration.unwrap();
            mpd.push(song)?;

        }
    }
    Ok(())
}

pub fn save_current_playlist(mpd: &mut mpd::Client) -> Result<(),Error>{
    mpd.pl_clear("temp")?;
    mpd.save("temp")?;
    Ok(())
}