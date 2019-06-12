use std::process::Command;
use std::io::copy;
use std::fs;
use std::time::Duration;
use std::sync::{Arc, Mutex, Condvar}

use std::path::Path;
use reqwest::{IntoUrl, get};

use crate::errors::Error;

fn download_file<U: IntoUrl>(url: U, save_path: &Path) -> Result<(), Error> {
    let mut response = reqwest::get(url)?;
    let mut dest = fs::File::create(save_path)?;
    copy(&mut response, &mut dest)?;
    Ok(())
}

#[derive(Clone)]
pub struct YoutubeDownloader {
    guard: Arc<(Mutex<bool>, Condvar)>,
}

impl YoutubeDownloader {
    pub fn init() -> Result<Self, Error>{
        let dir_path = Path::new("temp");
        let full_path = Path::new("temp/youtube-dl");
        
        if !full_path.exists() {
            if !dir_path.exists() {
                fs::create_dir(dir_path)?;
            }

            download_file("https://yt-dl.org/downloads/latest/youtube-dl", full_path)?;
        }
        Ok(Self {guard: Arc::new(Mutex::new(false), Condvar::new()) })
    }

    fn download(url: &str) -> Result<(), Error> {
        let full_path = Path::new("temp/youtube-dl");
        let output = Command::new(full_path)
                    .arg("-F")
                    .arg(url)
                    .output()?;

        dbg!(output);
        Ok(())
    }

    fn download_song_timeout(&mut self, url: &str, timeout: Duration) -> Result<(), Error> {
        
        let (free, waker) = self.guard.lock(x);
        if !free.lock.unwrap() {
            waker.wait_timeout(free, timeout)
        }
        

        let full_path = Path::new("temp/youtube-dl");
        let output = Command::new(full_path)
                    .arg("-F")
                    .arg(url)
                    .output()?;

        dbg!(output);

        waker.notify_one();
        Ok(())
    }

}