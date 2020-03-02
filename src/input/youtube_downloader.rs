use std::process::Command;
use std::io::Write;
use std::fs;
use std::os::unix::fs::OpenOptionsExt;
use std::thread;

use std::time::Duration;
use std::path::Path;

use reqwest::{IntoUrl};
use actix_rt;
use regex::Regex;
use serde::{Serialize, Deserialize};
use bincode;
use async_trait::async_trait;

use crate::input::bot::youtube_dl::TelegramFeedback;

const DIR: &str = "temp";
const YOUTUBE_DL_LOC: &str = "temp/youtube-dl";
const MUSIC_TEMP: &str = "temp/music";
const MUSIC_DIR: &str = "/home/pi/Music/youtube";

#[derive(Debug)]
pub enum Error{
    CouldNotDownloadExecutable(reqwest::Error),
    CouldNotStoreExecutable(std::io::Error),
    CouldNotDownloadSong(std::process::Output),
    CouldNotUpdateMpd(mpd::error::Error),
    CouldNotCreateTempDir(std::io::Error),
    UnexpectedYoutubeDlStdOut(String),
    UnsupportedSource(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::CouldNotDownloadExecutable(e) => 
                write!(f, "Could not download executable, cause {}", e),
            Error::CouldNotStoreExecutable(e) =>
                write!(f, "Could not store executable, cause: {}", e),
            Error::CouldNotDownloadSong(e) => 
                write!(f, "Could not download song, cause: {}", 
                String::from_utf8(e.stderr.clone()).unwrap()),
            Error::CouldNotUpdateMpd(e) =>
                write!(f, "Could not update mpd, cause: {}", e),
            Error::CouldNotCreateTempDir(e) => 
                write!(f, "Could not create temp directory, cause: {}", e),
            Error::UnexpectedYoutubeDlStdOut(s) => 
                write!(f, "Could not understand output of youtube-dl, output was: {}",s),
            Error::UnsupportedSource(s) => 
                write!(f, "Could not find something to download: {}", s),
        }
    }
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

#[async_trait]
pub trait Feedback {
    async fn feedback(&self, status: JobStatus, token: &str);
}

#[derive(Debug, Clone)]
pub enum FeedbackChannel{
    Telegram(TelegramFeedback),
    None,
    //future expand with ... signal?
}

#[derive(Debug)]
pub enum JobStatus {
    Finished,
    Downloaded,
    Queued(MetaData),
    Error,
}

impl FeedbackChannel {
    async fn send(self, status: JobStatus, token: &str) {
        match self {
            Self::Telegram(tel) => tel.feedback(status, token).await,
            Self::None => (),
        }
    }
}

struct Job {
    id: u64,
    url: String,
    feedback: FeedbackChannel,
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
    dbg!(&response.len());
    dest.write(&response)?;
    Ok(())
}

#[derive(Clone)]
pub struct YoutubeDownloader {
    url_tx: crossbeam_channel::Sender<Job>,
    db: sled::Db,
    token: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct MetaData {
    pub artist: String,
    pub title: String,
}

#[derive(Serialize, Deserialize, Default)]
struct DbEntry {
    downloaded: bool, //default val = false
    meta: Option<MetaData>
}

impl DbEntry {
    fn empty() -> Self {
        DbEntry {
            downloaded: false, 
            meta: None, 
        }
    }
    fn downloaded() -> Self {
        DbEntry {
            downloaded: true, 
            meta: None, 
        }
    }
}

fn get_meta(url: &str) -> Result<MetaData, Error> {

    let full_path = Path::new(YOUTUBE_DL_LOC);
    let output = Command::new(full_path)
        //.arg("--verbose") //for debugging
        .arg("--geo-bypass")
        .arg("--no-playlist")
        .arg("--ignore-config")
        .arg("--get-filename")

        .arg("--max-filesize") 
        .arg("50m")

        .arg("--audio-format")
        .arg("m4a")
        .arg("--audio-quality")
        .arg("257")
        .arg("--format")
        .arg("bestaudio[acodec=opus]/bestaudio/best")
        
        .arg("--output")
        .arg("/%(title)s-#-#-%(artist)s")
        .arg(url)
        .output()?;

    let stdout = String::from_utf8(output.stdout).unwrap();
    let mut title_artist = stdout.splitn(2, "-#-#-");
    let metadata = MetaData {
        artist: title_artist.next()
            .map(|s| s.to_owned())
            .ok_or(Error::UnsupportedSource(stdout.clone()))?,
        title: title_artist.next()
            .map(|s| s.to_owned())
            .ok_or(Error::UnsupportedSource(stdout.clone()))?,
    };

    Ok(metadata)
}

fn download_file(output_arg: &String, url: &str)
 -> std::io::Result<std::process::Output> {

    let full_path = Path::new(YOUTUBE_DL_LOC);
    Command::new(full_path)
        //.arg("--verbose") //for debugging
        .arg("--geo-bypass")
        .arg("--no-playlist")
        .arg("--ignore-config")

        .arg("--max-filesize") 
        .arg("50m")

        //.arg("--embed-thumbnail") //dependency can not be installed
        .arg("--write-thumbnail")
        .arg("--audio-format")
        .arg("m4a")
        .arg("--audio-quality")
        .arg("257")
        .arg("--postprocessor-args") //seems to take along time
        .arg("-ar 44100")
        .arg("--extract-audio")
        .arg("--format")
        .arg("bestaudio[acodec=opus]/bestaudio/best")

        .arg("--output")
        .arg(&output_arg)
        
        .arg(url)
        .output()
}

async fn download_song(url: &str, id: u64) -> Result<(), Error> {
    let temp_dir = std::path::PathBuf::from(MUSIC_TEMP);
    if !temp_dir.exists() {
        fs::create_dir(&temp_dir)
            .map_err(|e| Error::CouldNotCreateTempDir(e))?;
    }

    let mut output_arg = String::from(temp_dir.to_str().unwrap());
    output_arg.push_str(&format!("/{}.%(ext)s",id));

    let output = download_file(&output_arg, url)?;
    dbg!(&output);

    let output = if !output.status.success() {
        let since_last_updated = fs::metadata(YOUTUBE_DL_LOC).unwrap()
            .created().unwrap()
            .elapsed().unwrap_or(Duration::from_secs(0));
        
        // updating youtube-dl wont do anything return error
        if since_last_updated > Duration::from_secs(60*60*24){
            return Err(Error::CouldNotDownloadSong(output));
        } 
        
        download_youtube_dl("https://yt-dl.org/downloads/latest/youtube-dl").await?;
        let output = download_file(&output_arg, url)?;
        if !output.status.success() {
            return Err(Error::CouldNotDownloadSong(output));
        }
        output
    } else {output};

    //let title = parse_stdout(output.stdout)?;
    
    //mpd::Client::connect("127.0.0.1:6600").and_then(|mut c| c.rescan())?;
    //Ok(title)
    Ok(())
}

fn get_metadata(db: sled::Tree, id: u64)
 -> Result<Option<MetaData>, Error> {

    let modification = db.compare_and_swap(
        id.to_be_bytes(), 
        Some(bincode::serialize(&DbEntry::empty()).unwrap() ),
        Some(bincode::serialize(&DbEntry::downloaded()).unwrap() ),
    ).unwrap();
    
    //if the modification was an error (not swapped) the entry was not
    //empty and thus has metadata, return the metadata. 
    if let Err(old_value) = modification {
        let with_metadata: DbEntry = bincode::deserialize(
            &old_value
            .current.unwrap()
            .to_vec() ).unwrap();
        Ok(with_metadata.meta)
    } else {
        Ok(None)
    }
}

fn write_metadata(meta: MetaData, id: u64) -> Result<(), Error> {
    //move file from temp/id to music/artist/title
    //or in case metadata is wierd (tbdefined) to
    //music/youtube/title.. 
    todo!();
}

async fn handle_job(job: Job, token: String, db: sled::Tree) 
    -> Result<(), Error> {
    
    let status = match download_song(&job.url, job.id).await {
        Ok(_) => {
            if let Some(meta) = get_metadata(db, job.id)?{
                write_metadata(meta, job.id)?;
                JobStatus::Finished
            } else {
                JobStatus::Downloaded
            }
        },
        Err(error) => {
            error!("warn error during song download: {:?}", error);
            JobStatus::Error
        },
    };
    job.feedback.send(status, &token).await;
    Ok(())
}

fn song_downloader(url_rx: crossbeam_channel::Receiver<Job>, 
    token: String, db: sled::Tree) {
    
    let mut rt = actix_rt::Runtime::new().unwrap();
    
    loop {
        match url_rx.recv() {
            Ok(job) => {
                if let Err(e) = rt.block_on( handle_job(job, token.clone(), db.clone())){
                    error!("could not handle download job: {:?}", e);
                }
            },
            // Err means YoutubeDownloader 
            // was dropped and this thread should stop
            Err(_) => return,
        }
    }
}


impl YoutubeDownloader {
    pub async fn init(token: String, db: sled::Db)
     -> Result<(Self, thread::JoinHandle<()>), Error> {

        let youtube_db = db.open_tree("youtube_dl").unwrap();
        let full_path = Path::new(YOUTUBE_DL_LOC);
        let dir_path = Path::new(DIR);

        if !full_path.exists() {
            if !dir_path.exists() {
                fs::create_dir(dir_path)?;
            }
            download_youtube_dl("https://yt-dl.org/downloads/latest/youtube-dl").await?;
        }

        let (url_tx, url_rx) = crossbeam_channel::bounded(10);
        let local_token = token.clone();
        let downloader_thread = thread::spawn(move || 
            song_downloader(url_rx, local_token, youtube_db)
        );

        Ok((Self {url_tx, db, token}, downloader_thread))
    }

    //TODO use youtube downloader to stream audio from youtube!!!

    /// checks if a song can be downloaded, if so sends the command to start download to 
    /// a seperate thread
    pub async fn add_song_to_queue(&self, url: String, 
        feedback: FeedbackChannel) -> Result<(), Error> {
        
        //get metadata guess from youtube-dl
        let meta_guess = get_meta(&url)?;
        let id = self.db.generate_id().unwrap();
        let youtube_db = self.db.open_tree("youtube_dl").unwrap();
        
        let db_entry = DbEntry::default();
        youtube_db.insert(
            id.to_be_bytes(), 
            bincode::serialize(&db_entry).unwrap()
        ).unwrap();

        let job = Job {id, url: url, feedback: feedback.clone()};
        self.url_tx.try_send(job).unwrap();
        feedback.send(JobStatus::Queued(meta_guess), &self.token).await;
        dbg!();
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    /*#[test]
    fn test_parse_stdout(){

        let test_case = String::from("[youtube] 1MZu2pD3QPY: Downloading webpage\n[youtube] 1MZu2pD3QPY: Downloading video info webpage\n[youtube] Downloading just video 1MZu2pD3QPY because of --no-playlist\n[download] Destination: /home/pi/Music/youtube/Trevor Jones - Promentory (The Last of the Mohicans).webm\n\r[download]   0.0% of 6.03MiB at 527.45KiB/s ETA 00:12\r[download]   0.0% of 6.03MiB at  1.06MiB/s ETA 00:05\r[download]   0.1% of 6.03MiB at  1.90MiB/s ETA 00:03\r[download]   0.2% of 6.03MiB at  3.33MiB/s ETA 00:01\r[download]   0.5% of 6.03MiB at  4.79MiB/s ETA 00:01\r[download]   1.0% of 6.03MiB at  6.59MiB/s ETA 00:00\r[download]   2.1% of 6.03MiB at  7.89MiB/s ETA 00:00\r[download]   4.1% of 6.03MiB at  9.44MiB/s ETA 00:00\r[download]   8.3% of 6.03MiB at 10.43MiB/s ETA 00:00\r[download]  16.6% of 6.03MiB at 10.76MiB/s ETA 00:00\r[download]  33.1% of 6.03MiB at 10.06MiB/s ETA 00:00\r[download]  66.3% of 6.03MiB at 10.36MiB/s ETA 00:00\r[download] 100.0% of 6.03MiB at 10.33MiB/s ETA 00:00\r[download] 100% of 6.03MiB in 00:00\n[fromtitle] parsed artist: Trevor Jones\n[fromtitle] parsed title: Promentory (The Last of the Mohicans)\n[ffmpeg] Destination: /home/pi/Music/youtube/Trevor Jones - Promentory (The Last of the Mohicans).m4a\nDeleting original file /home/pi/Music/youtube/Trevor Jones - Promentory (The Last of the Mohicans).webm (pass -k to keep)\n[ffmpeg] Adding metadata to \'/home/pi/Music/youtube/Trevor Jones - Promentory (The Last of the Mohicans).m4a\'\n");
        let test_case = test_case.into_bytes();
        let test_case_awnser = "Trevor Jones - Promentory (The Last of the Mohicans)";
        assert_eq!(parse_stdout(test_case).unwrap(), test_case_awnser);
    }*/
}