use std::process::Command;
use std::io::Write;
use std::fs;
use std::os::unix::fs::OpenOptionsExt;
use std::thread;

use std::time::Duration;
use std::path::{Path, PathBuf};

use reqwest::{IntoUrl};
use actix_rt;
use serde::{Serialize, Deserialize};
use bincode;
use async_trait::async_trait;
use async_std::task;
use sled::Batch;
use std::ffi::OsStr;
use regex::Regex;
//use id3;

use crate::input::bot::youtube_dl::TelegramFeedback;

const DIR: &str = "temp";
const YOUTUBE_DL_LOC: &str = "temp/youtube-dl";
const MUSIC_TEMP: &str = "temp/music";
const MUSIC_DIR: &str = "/home/pi/Music";

#[derive(Debug)]
pub enum Error{
    CouldNotDownloadExecutable(reqwest::Error),
    CouldNotStoreExecutable(std::io::Error),
    CouldNotDownloadSong(std::process::Output),
    CouldNotUpdateMpd(mpd::error::Error),
    CouldNotCreateTempDir(std::io::Error),
    UnexpectedYoutubeDlStdOut(String),
    UnsupportedSource(String),
    CanNoLongerUpdateMeta,
    CanNotSwapArtistWithEmptyTitle,
    IDWasDeleted,
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
            Error::CanNoLongerUpdateMeta =>
                write!(f, "Can no longer update metadata, was already done some time ago"),
            Error::CanNotSwapArtistWithEmptyTitle => 
                write!(f, "Can not swap artist and title as the current artist \
                is empty and the title can not be empty. Not writing metadata"),
            Error::IDWasDeleted => 
                write!(f, "Internal error, file not in database"),
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
    Queued(MetaGuess, u64),
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
    pub source_url: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct MetaGuess {
    pub artist: String,
    pub title: String,
    pub source_url: String,
}

/*impl MetaData {
    fn to_tag(&self) -> id3::Tag {
        let mut tag = id3::Tag::new();
        tag.set_artist(&self.artist);
        tag.set_title(&self.title);
        //tag.set_text("source: ",&self.source_url);
        tag
    }
}*/

// in the future queued an downloaded can be expanded
// with a MetaGuess
#[derive(Serialize, Deserialize)] 
enum Status {
    Queued(MetaGuess), //can go to Downloaded or ConfirmedMeta
    Downloaded(MetaGuess), // can go to Metawritten (as soon as there is meta we write it to file)
    MetaConfirmed(MetaData), //can go to MetaWritten (as soon as download is complete meta written to file)
    MetaWritten((PathBuf, MetaData)), //can only tranfrom to WritingMeta
}

fn split(to_split_on: &str) -> (&str, &str){
    //try splitting on title
    let new_title;
    let new_artist;

    let in_quotes = Regex::new(r#"(.*)\s*(?:'|")\s*(.*)\s*(?:'|")\s*(.*)"#).unwrap();
    if to_split_on.contains("-"){
        let mut split = to_split_on.splitn(2,"-");
        new_title = split.next().unwrap();
        new_artist = split.next().unwrap();
    } else if to_split_on.contains(":"){
        let mut split = to_split_on.splitn(2,":");
        new_title = split.next().unwrap();
        new_artist = split.next().unwrap();
    } else if to_split_on.contains("|"){
        let mut split = to_split_on.splitn(2,"|");
        new_title = split.next().unwrap();
        new_artist = split.next().unwrap();
    } else if let Some(caps) = in_quotes.captures(to_split_on){
        dbg!(&caps);
        let len_before = caps.get(1).unwrap().as_str().len();
        let len_after = caps.get(3).unwrap().as_str().len();
        if len_before > len_after {
            new_title = caps.get(1).unwrap().as_str();
            new_artist = caps.get(2).unwrap().as_str();
        } else {
            new_title = caps.get(2).unwrap().as_str();
            new_artist = caps.get(3).unwrap().as_str();
        }
    } else {
        new_title = to_split_on;
        new_artist = "unknown";
    }

    (new_title.trim(), new_artist.trim())
}


fn guess_meta(url: &str) -> Result<MetaGuess, Error> {

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
        .arg("%(title)s")
        .arg(url)
        .output()?;

    let stdout = String::from_utf8(output.stdout).unwrap();
    dbg!(&stdout);
    let (artist, title) = split(&stdout);

    Ok(MetaGuess {
        artist: artist.to_owned(), 
        title: title.to_owned(),
        source_url: url.to_owned(),
    })
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

    if !output.status.success() {
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
    }
    Ok(())
}

fn ffmpeg_set_meta(input: &Path, output: &Path, artist: &str, title: &str){
    let input = fs::canonicalize(input).unwrap();
    dbg!(&input);
    dbg!(&output);

    let output = Command::new("ffmpeg")
        .args(&[OsStr::new("-i"),input.as_os_str()])
        .args(&["-metadata", &format!("artist={}",artist)])
        .args(&["-metadata", &format!("title={}",title)])
        .args(&["-codec", "copy"])
        .arg(output.as_os_str())
        .output()
        .expect("ffmpeg write metadata");

    dbg!(output);
    //ffmpeg -i InputFile -metadata key=value -codec copy OutputFile
}

fn write_metadata(id: u64, meta: &MetaData) -> Result<PathBuf, Error> {
    let mut old_path = PathBuf::from(MUSIC_TEMP);
    old_path.push(id.to_string());
    old_path.set_extension("m4a");

    let mut new_path = PathBuf::from(MUSIC_DIR);
    new_path.push(&meta.artist);
    if !new_path.exists() {
        fs::create_dir(&new_path)?;
    }
    new_path.push(&meta.title);
    new_path.set_extension("m4a");

    ffmpeg_set_meta(&old_path, &new_path, &meta.artist, &meta.title);
    fs::remove_file(old_path).unwrap();
    mpd::Client::connect("127.0.0.1:6600").and_then(|mut c| c.rescan())?;
    Ok(new_path)
}

fn update_metadata(old_path: &Path, meta: &MetaData) -> Result<PathBuf, Error> {
    let mut new_path = PathBuf::from(MUSIC_DIR);
    new_path.push(&meta.artist);
    if !new_path.exists() {
        fs::create_dir(&new_path)?;
    }
    new_path.push(&meta.title);
    new_path.set_extension("m4a");
    
    ffmpeg_set_meta(&old_path, &new_path, &meta.artist, &meta.title);
    
    fs::remove_file(old_path).unwrap();
    //delete if parent dir is now empty
    if let Some(dir) = old_path.parent(){
        if dir.read_dir().unwrap().count() == 0 {
            fs::remove_dir(dir).unwrap();
        }
    }

    mpd::Client::connect("127.0.0.1:6600").and_then(|mut c| c.rescan())?;
    Ok(new_path)
}

async fn handle_job(job: Job, token: String, db: sled::Tree) 
    -> Result<(), Error> {
    
    let status = match download_song(&job.url, job.id).await {
        Ok(_) => { 
            aquire_db_mutex(&db, job.id).await;
            let db_status = db.get(&status_key(job.id))
                .unwrap()
                .ok_or(Error::CanNoLongerUpdateMeta)?;
            let db_status = bincode::deserialize(&db_status.to_vec()).unwrap();
            let (new_db_status, job_status) = match db_status {
                Status::Queued(meta_guess) => {
                    (Status::Downloaded(meta_guess), JobStatus::Downloaded)
                },
                Status::MetaConfirmed(meta) => {
                    let new_path = write_metadata(job.id, &meta)?; //write meta
                    (Status::MetaWritten((new_path, meta)), JobStatus::Finished)
                },
                Status::Downloaded(_) => {
                    panic!("file should not yet be download!")
                },
                Status::MetaWritten((_, _)) => {
                    panic!("file should not yet be download!")
                },
            };
            
            let mut batch = Batch::default();
            batch.insert(
                &status_key(job.id), 
                bincode::serialize(&new_db_status).unwrap()
            );
            batch.insert( //unlock file lock
                &db_mutex_key(job.id),
                bincode::serialize(&false).unwrap()
            );
            db.apply_batch(batch).unwrap();
            job_status
        },
        Err(error) => {
            error!("warn error during song download: {:?}", error);
            //TODO remove from database
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

async fn aquire_db_mutex(db: &sled::Tree, id: u64){
    while let Err(_is_locked) = db.compare_and_swap(
        &db_mutex_key(id),
        Some(bincode::serialize(&false).unwrap()),
        Some(bincode::serialize(&true).unwrap())).unwrap(){
        task::sleep(Duration::from_millis(10)).await;
        dbg!("blocking on mutex aquisition");
    }
}

fn db_mutex_key(id: u64) -> [u8;9]{
    let mut a = [0;9];
    a[..8].clone_from_slice(&id.to_be_bytes());
    return a;
}
fn status_key(id: u64) -> [u8;9]{
    let mut a = [1;9];
    a[..8].clone_from_slice(&id.to_be_bytes());
    return a;
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
        let meta_guess = guess_meta(&url)?;
        let id = self.db.generate_id().unwrap();
        let db = self.db.open_tree("youtube_dl").unwrap();
         
        let db_entry = Status::Queued(meta_guess.clone());
        let mut batch = Batch::default();
        batch.insert(
            &status_key(id), 
            bincode::serialize(&db_entry).unwrap()
        );
        batch.insert(
            &db_mutex_key(id),
            bincode::serialize(&false).unwrap()
        );
        db.apply_batch(batch).unwrap();

        let job = Job {id, url: url, feedback: feedback.clone()};
        self.url_tx.try_send(job).unwrap();
        feedback.send(JobStatus::Queued(meta_guess, id), &self.token).await;
        dbg!();
        Ok(())
    }

    pub async fn meta_ok(&self, id: u64)
     -> Result<(), Error> {
    
        let db = self.db.open_tree("youtube_dl").unwrap();

        aquire_db_mutex(&db, id).await;
        let job_status = db.get(&status_key(id))
            .unwrap()
            .ok_or(Error::CanNoLongerUpdateMeta)?;
        let job_status = bincode::deserialize(&job_status.to_vec()).unwrap();
        let new_status = match job_status {
            Status::Queued(meta_guess) => {
                let new_meta = MetaData {
                    title: meta_guess.title,
                    artist: meta_guess.artist,
                    source_url: meta_guess.source_url,
                };
                Status::MetaConfirmed(new_meta)
            },
            Status::MetaConfirmed(meta) => Status::MetaConfirmed(meta),
            Status::Downloaded(meta_guess) => {
                let new_meta = MetaData {
                    title: meta_guess.title,
                    artist: meta_guess.artist,
                    source_url: meta_guess.source_url,
                };
                let new_path = write_metadata(id, &new_meta)?;
                Status::MetaWritten((new_path, new_meta))
            },
            Status::MetaWritten(i) => Status::MetaWritten(i),
        };
        
        let mut batch = Batch::default();
        batch.insert(
            &status_key(id), 
            bincode::serialize(&new_status).unwrap()
        );
        batch.insert( //unlock file lock
            &db_mutex_key(id),
            bincode::serialize(&false).unwrap()
        );
        db.apply_batch(batch).unwrap();       
        Ok(())
    }

    pub async fn swap_meta(&self, id: u64)
     -> Result<(), Error> {
    
        let db = self.db.open_tree("youtube_dl").unwrap();

        aquire_db_mutex(&db, id).await;
        let job_status = db.get(&status_key(id))
            .unwrap()
            .ok_or(Error::CanNoLongerUpdateMeta)?;
        let job_status = bincode::deserialize(&job_status.to_vec()).unwrap();
        let new_status = match job_status {
            Status::Queued(meta_guess) => {
                let new_meta = MetaData {
                    title: meta_guess.artist,
                    artist: meta_guess.title,
                    source_url: meta_guess.source_url,
                };
                Status::MetaConfirmed(new_meta)
            },
            Status::MetaConfirmed(mut meta) => {
                meta.title = meta.artist.clone();
                meta.artist = meta.title.clone();
                Status::MetaConfirmed(meta)
            },
            Status::Downloaded(meta_guess) => {
                let new_meta = MetaData {
                    title: meta_guess.artist,
                    artist: meta_guess.title,
                    source_url: meta_guess.source_url,
                };
                let new_path = write_metadata(id, &new_meta)?;
                Status::MetaWritten((new_path, new_meta))
            },
            Status::MetaWritten((path, mut meta)) => {
                meta.artist = meta.title.clone();
                meta.title = meta.artist.clone();
                let new_path = update_metadata(&path, &meta)?;
                Status::MetaWritten((new_path, meta))
            },
        };
        
        let mut batch = Batch::default();
        batch.insert(
            &status_key(id), 
            bincode::serialize(&new_status).unwrap()
        );
        batch.insert( //unlock file lock
            &db_mutex_key(id),
            bincode::serialize(&false).unwrap()
        );
        db.apply_batch(batch).unwrap();       
        Ok(())
    }

    pub async fn no_meta(&self, id: u64)
     -> Result<(), Error> {
    
        let db = self.db.open_tree("youtube_dl").unwrap();

        aquire_db_mutex(&db, id).await;
        let job_status = db.get(&status_key(id))
            .unwrap()
            .ok_or(Error::CanNoLongerUpdateMeta)?;
        let job_status = bincode::deserialize(&job_status.to_vec()).unwrap();
        let new_status = match job_status {
            Status::Queued(meta) => {
                let new_meta = MetaData {
                    title: format!("{} {}",meta.title,meta.artist),
                    artist: "".to_owned(),
                    source_url: meta.source_url,
                };
                Status::MetaConfirmed(new_meta)
            },
            Status::MetaConfirmed(mut meta) => {
                meta.title = format!("{} {}",meta.title,meta.artist);
                meta.artist = "".to_owned();
                Status::MetaConfirmed(meta)
            },
            Status::Downloaded(meta) => {
                let new_meta = MetaData {
                    title: format!("{} {}",meta.title,meta.artist),
                    artist: "".to_owned(),
                    source_url: meta.source_url,
                };
                let new_path = write_metadata(id, &new_meta)?;
                Status::MetaWritten((new_path, new_meta))
            },
            Status::MetaWritten((path, mut meta)) => {
                meta.title = format!("{} {}",meta.title,meta.artist);
                meta.artist = "".to_owned();
                let new_path = update_metadata(&path, &meta)?;
                Status::MetaWritten((new_path, meta))
            },
        };
        
        let mut batch = Batch::default();
        batch.insert(
            &status_key(id), 
            bincode::serialize(&new_status).unwrap()
        );
        batch.insert( //unlock file lock
            &db_mutex_key(id),
            bincode::serialize(&false).unwrap()
        );
        db.apply_batch(batch).unwrap();
        
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

    #[test]
    fn test_split(){
        let (artist, title) = split("Merlin 4 Soundtrack \"The Burial\" 04");
        assert_eq!(artist, "Merlin 4 Soundtrack");
        assert_eq!(title, "The Burial");
    }
    
    
}