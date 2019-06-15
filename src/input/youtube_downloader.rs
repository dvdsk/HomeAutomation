use std::process::Command;
use std::io::copy;

use std::fs;
use std::os::unix::fs::OpenOptionsExt;

use std::time::Duration;
use std::thread;
use crossbeam_channel::{unbounded, Receiver, Sender};

use std::path::Path;
use reqwest::{IntoUrl, get};

use crate::errors::Error;

fn download_file<U: IntoUrl>(url: U, save_path: &Path) -> Result<(), Error> {
    
    let mut dest = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o770)
        .open(save_path)?;

    let mut response = reqwest::get(url)?;    
    copy(&mut response, &mut dest)?;
    
    Ok(())
}

#[derive(Clone)]
pub struct YoutubeDownloader {
    url_tx: crossbeam_channel::Sender<String>,
}

fn download_song(url: &str) -> Result<(), Error> {
    //let mut music_dir = dirs::home_dir().unwrap();
    //music_dir.push("Music");
    let music_dir = std::path::PathBuf::from("/home/pi/Music");
    let mut output_arg = String::from(music_dir.to_str().unwrap());
    output_arg.push_str("/%(title)s.%(ext)s");

    let full_path = Path::new("temp/youtube-dl");
    let output = Command::new(full_path)
                .arg("--format")
                .arg("bestaudio/best")
                .arg("--extract-audio")
                .arg("--output")
                .arg(&output_arg)
                //.arg("--embed-thumbnail")
                .arg(url)
                .output()?;

    dbg!(&output);
    if output.status.success() {

        //TODO move to mpd music location
        mpd::Client::connect("127.0.0.1:6600").and_then(|mut c| c.rescan())?;
    } else {
        dbg!("HANDLE ERROR");
    }

    Ok(())
}

fn song_downloader(url_rx: crossbeam_channel::Receiver<String>) {
    loop {
        match url_rx.recv() {
            Ok(url) => {
                dbg!(&url);
                match download_song(&url) {
                    Ok(_) => dbg!(),
                    Err(error) => error!("error during song download: {:?}", error),
                };
            },
            // return without url means YoutubeDownloader was dropped and we should stop
            Err(_) => return,
        }   
    }    
}


impl YoutubeDownloader {
    pub fn init() -> Result<(Self, thread::JoinHandle<()>), Error>{
        let dir_path = Path::new("temp");
        let full_path = Path::new("temp/youtube-dl");
        
        if !full_path.exists() {
            if !dir_path.exists() {
                fs::create_dir(dir_path)?;
            }
            download_file("https://yt-dl.org/downloads/latest/youtube-dl", full_path)?;
        }

        let (url_tx, url_rx) = crossbeam_channel::bounded(10);
        let downloader_thread = thread::spawn(move || song_downloader(url_rx) );

        Ok((Self {url_tx}, downloader_thread))
    }

    //TODO use youtube downloader to stream audio from youtube!!!

    /// checks if a song can be downloaded, if so sends the command to start download to 
    /// a seperate thread
    pub fn add_song_to_queue(&self, url: String) -> Result<(), ()> {
        
        if self.url_tx.try_send(url).is_err(){
            error!("could not add song to downlaod list");
            Err(())
        } else {
            dbg!();
            Ok(())
        }
    }

}