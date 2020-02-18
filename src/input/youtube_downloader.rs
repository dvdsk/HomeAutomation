use std::process::Command;
use std::io::Write;
use std::fs;
use std::os::unix::fs::OpenOptionsExt;
use std::thread;

use std::time::Duration;
use std::path::Path;
use reqwest::{IntoUrl};
use actix_rt;

const DIR: &str = "temp";
const YOUTUBE_DL_LOC: &str = "temp/youtube-dl";

#[derive(Debug)]
pub enum Error{
    CouldNotDownloadExecutable(reqwest::Error),
    CouldNotStoreExecutable(std::io::Error),
    CouldNotDownloadSong,
    CouldNotUpdateMpd(mpd::error::Error),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::CouldNotDownloadExecutable(err)
    }
}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::CouldNotStoreExecutable(err)
    }
}
impl From<mpd::error::Error> for Error {
    fn from(err: mpd::error::Error) -> Self {
        Error::CouldNotUpdateMpd(err)
    }
}

async fn download_youtube_dl<U: IntoUrl>(url: U) -> Result<(), Error> {
    
    let mut dest = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o770)
        .open(YOUTUBE_DL_LOC)?;

    let response = reqwest::get(url)
        .await?
        .bytes()
        .await?;
    
    dest.write(&response)?;
    Ok(())
}

#[derive(Clone)]
pub struct YoutubeDownloader {
    url_tx: crossbeam_channel::Sender<String>,
}

fn run_youtube_dl(output_arg: &String, url: &str)
 -> std::io::Result<std::process::Output> {

    let full_path = Path::new(YOUTUBE_DL_LOC);
    Command::new(full_path)
        .arg("--geo-bypass")
        .arg("--ignore-config")
        .arg("--add-metadata")
        .arg("--metadata-from-title \"%(artist)s - %(title)s\"")

        .arg("--embed-thumbnail")
        .arg("--audio-format")
        .arg("m4a")
        .arg("--audio-quality")
        .arg("257")
        .arg("--postprocessor-args")
        .arg("\"-ar 44100\"")
        .arg("--extract-audio")
        .arg("--format")
        .arg("bestaudio[acodec=opus]/bestaudio/best")

        .arg(&output_arg)
        .arg(url)
        .output()
}

fn download_song(url: &str) -> Result<(), Error> {
    let music_dir = std::path::PathBuf::from("/home/pi/Music");
    let mut output_arg = String::from(music_dir.to_str().unwrap());
    output_arg.push_str("/%(title)s.%(ext)s");

    let output = run_youtube_dl(&output_arg, url)?;
    dbg!(&output);

    if !output.status.success() {
        let since_last_updated = fs::metadata(YOUTUBE_DL_LOC).unwrap()
            .created().unwrap()
            .elapsed().unwrap_or(Duration::from_secs(0));
        
        if since_last_updated > Duration::from_secs(60*60*24){
            return Err(Error::CouldNotDownloadSong);
        } 
        
        
        let mut rt = actix_rt::Runtime::new().unwrap();
        rt.block_on(
            download_youtube_dl("https://yt-dl.org/downloads/latest/youtube-dl")
        )?;
        
        let output = run_youtube_dl(&output_arg, url)?;
        dbg!(&output);

        if !output.status.success() {
            return Err(Error::CouldNotDownloadSong);
        }
    }

    mpd::Client::connect("127.0.0.1:6600").and_then(|mut c| c.rescan())?;
    Ok(())
}

fn song_downloader(url_rx: crossbeam_channel::Receiver<String>) {
    loop {
        match url_rx.recv() {
            Ok(url) => {
                dbg!(&url);
                match download_song(&url) {
                    Ok(_) => dbg!(),
                    Err(error) => {
                        error!("warn error during song download: {:?} \
                        trying with updated youtube-dl", error);
                    },
                };
            },
            // Err means YoutubeDownloader 
            // was dropped and this thread should stop
            Err(_) => return,
        }   
    }    
}


impl YoutubeDownloader {
    pub async fn init() -> Result<(Self, thread::JoinHandle<()>), Error>{
        let full_path = Path::new(YOUTUBE_DL_LOC);
        let dir_path = Path::new(DIR);

        if !full_path.exists() {
            if !dir_path.exists() {
                fs::create_dir(dir_path)?;
            }
            download_youtube_dl("https://yt-dl.org/downloads/latest/youtube-dl").await?;
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