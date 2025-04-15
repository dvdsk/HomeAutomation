#![allow(clippy::enum_glob_use)]

use std::fmt;
use std::time::Duration;

use jiff::civil::Time;
use mpdrs::status::State;
use mpdrs::Playlist;
use mpdrs::{error::Error, Song};
use rand::seq::IndexedRandom;
use tracing::{debug, info, instrument, trace};

use db::Db;
use mpdinterface::MpdInterface;

mod db;
mod db2;
mod mpdinterface;

#[derive(Debug)]
enum Direction {
    Next,
    Previous,
}

#[allow(clippy::struct_excessive_bools)]
struct Settings {
    repeat: bool,
    random: bool,
    single: bool,
    consume: bool,
    volume: i8,

    save_playlist: bool,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, Clone)]
pub enum AudioMode {
    Music,
    Singing,
    Podcast,
    Meditation,
}

impl AudioMode {
    fn next(&mut self) {
        use AudioMode::*;
        *self = match self {
            Music => Singing,
            Singing => Podcast,
            Podcast => Meditation,
            Meditation => Music,
        }
    }

    fn to_prefix(&self) -> &str {
        use AudioMode::*;
        match self {
            Music => "music_",
            Singing => "singing_",
            Podcast => "podcast_",
            Meditation => "meditation_",
        }
    }

    fn settings(&self) -> Settings {
        use AudioMode::*;
        match self {
            Music | Singing => Settings {
                repeat: false,
                random: false,
                single: false,
                consume: false,
                volume: 70,
                save_playlist: false,
            },
            Podcast => Settings {
                repeat: false,
                random: false,
                single: false,
                consume: true,
                volume: 100,
                save_playlist: true,
            },
            Meditation => Settings {
                repeat: false,
                random: false,
                consume: false,
                single: true,
                volume: 100,
                save_playlist: false,
            },
        }
    }

    fn to_bytes(&self) -> [u8; 1] {
        use AudioMode::*;
        match self {
            Music => [1],
            Singing => [2],
            Podcast => [3],
            Meditation => [4],
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        use AudioMode::*;
        match bytes {
            &[1] => Music,
            &[2] => Singing,
            &[3] => Podcast,
            &[4] => Meditation,
            other => panic!("Unexpected serialized mode: {:?}", other),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ForceRewind {
    Yes,
    No,
}

pub struct AudioController {
    ip: String,
    port: String,
    client: MpdInterface,
    db: Db,
    pub(crate) mode: AudioMode,
}

impl fmt::Debug for AudioController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioController")
            .field("mode", &self.mode)
            .finish()
    }
}

impl AudioController {
    pub fn new(ip: &str, port: &str) -> Self {
        let address = format!("{}:{}", ip, port);
        let client = MpdInterface::connect(&address).unwrap();
        let mut controller = AudioController {
            ip: ip.to_owned(),
            port: port.to_owned(),
            client,
            db: Db::open("database/small_bedroom_audio"),
            mode: AudioMode::Music,
        };

        if let Some(mode) = controller.fetch_current_mode() {
            info!("Initializing with stored mode {:?}", mode);
            controller.mode = mode;
        } else {
            info!("No current mode stored, defaulting to music");
        }

        controller.playing();
        controller
    }

    pub fn reconnect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let address = format!("{}:{}", self.ip, self.port);
        self.client = MpdInterface::connect(&address)?;
        Ok(())
    }

    pub fn rescan(&mut self) {
        info!("Rescanning mpd library");
        self.client.rescan().unwrap();
    }

    pub fn playing(&mut self) -> bool {
        let playback_state = self.client.status().unwrap().state;
        playback_state == State::Play
    }

    fn stopped(&mut self) -> bool {
        let playback_state = self.client.status().unwrap().state;
        playback_state == State::Stop
    }

    fn get_playlists(&mut self) -> Vec<Playlist> {
        self.client.playlists().unwrap()
    }

    #[instrument(ret)]
    fn auto_rewind_time(last_played: u64) -> Duration {
        const MIN_REWIND: u32 = 2;

        let since_last_played = Db::now_timestamp() - last_played;
        info!("{}s since last played", since_last_played);

        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let rewind_time = (0.5 * (since_last_played as f64).sqrt())
            .round()
            .clamp(0.0, 30.0) as u32;

        if rewind_time < MIN_REWIND {
            Duration::from_secs(0)
        } else {
            Duration::from_secs(rewind_time.into())
        }
    }

    fn store_current_pausing(&self) {
        if let Some(current_playlist) = self.db.fetch_playlist_name(&self.mode)
        {
            self.db
                .store_last_played(&current_playlist, Db::now_timestamp());
        }
    }

    fn rewind_after_pause(&mut self) {
        use AudioMode::*;
        const SONG_RESTART_THRESHOLD: Duration =
            Duration::from_secs(4 * 60 * 60 * 10000);
        const ALMOST_OVER: Duration = Duration::from_secs(30);

        let current_playlist = match self.db.fetch_playlist_name(&self.mode) {
            Some(current_playlist) => current_playlist,
            None => return,
        };

        let last_played = self.db.fetch_last_played(&current_playlist);

        match (&self.mode, last_played) {
            (Podcast, Some(last_played)) => {
                self.rewind_by(Self::auto_rewind_time(last_played));
            }
            (Music | Singing | Meditation, Some(last_played)) => {
                if let (Some(length), Some(position)) =
                    (self.get_song_length(), self.get_elapsed())
                {
                    debug!(
                        "Song length: {length:?}, song position: {position:?}"
                    );
                    let time_left = length - position;
                    if Duration::from_secs(Db::now_timestamp() - last_played)
                        > SONG_RESTART_THRESHOLD
                        && time_left > ALMOST_OVER
                    {
                        self.seek_in_cur(0);
                    }
                }
            }
            (_, None) => (),
        }
    }

    #[instrument]
    pub fn toggle_playback(&mut self) {
        info!("Toggle playback");
        let was_playing = self.playing();

        if self.stopped() {
            self.client.play().unwrap();
        } else {
            self.client.toggle_pause().unwrap();
        }

        if was_playing {
            self.store_current_pausing();
        } else {
            self.rewind_after_pause();
        }
    }

    #[instrument]
    pub fn play(&mut self, force_rewind: ForceRewind) {
        if !self.playing() {
            self.toggle_playback();
        } else if force_rewind == ForceRewind::Yes {
            self.rewind_after_pause()
        }
    }

    fn get_song_length(&mut self) -> Option<Duration> {
        self.client.status().unwrap().duration
    }

    fn get_elapsed(&mut self) -> Option<Duration> {
        self.client.status().unwrap().elapsed
    }

    /// # Panics
    ///
    /// Panics if new position is over 4,294,967,295 seconds into the song,
    /// which is 136 years. I assume this will never happen.
    ///
    /// Panics if client.rewind() returns an error. This may very well happen.
    fn rewind_by(&mut self, duration: Duration) {
        if duration == Duration::from_secs(0) {
            debug!("0 seconds, not rewinding");
            return;
        }
        info!("Rewinding by {:?}", duration);

        if let Some(position) = self.get_elapsed() {
            self.client
                .rewind(
                    position
                        .saturating_sub(duration)
                        .as_secs()
                        .try_into()
                        .unwrap(),
                )
                .unwrap();
        }
    }

    pub fn rewind(&mut self) {
        self.rewind_by(Duration::from_secs(15));
        self.play(ForceRewind::No);
    }

    /// # Panics
    ///
    /// Panics if new position is over 4,294,967,295 seconds into the song,
    /// which is 136 years. I assume this will never happen.
    ///
    /// Panics if client.rewind() returns an error. This may very well happen.
    pub fn skip(&mut self) {
        info!("Skipping by 15 seconds");

        if let Some(position) = self.get_elapsed() {
            self.client
                .rewind((position.as_secs() + 15).try_into().unwrap())
                .unwrap();
        }

        self.play(ForceRewind::No);
    }

    pub fn previous(&mut self) {
        info!("Going to previous track");

        match self.client.prev() {
            Ok(_) => (),
            Err(Error::Server(server_error)) => {
                info!("Ignoring error during prev: {server_error}");
                assert!(
                    server_error.detail == "Not playing",
                    "Unexpected ServerError: {server_error}"
                );
            }
            Err(other_error) => panic!("Unexpected error: {other_error}"),
        };
        // self.play(ForceRewind::No);
    }

    #[instrument]
    pub fn next(&mut self) {
        info!("Next");

        match self.client.next() {
            Ok(_) => (),
            Err(Error::Server(server_error)) => {
                assert!(
                    server_error.detail == "Not playing",
                    "Unexpected ServerError: {server_error}"
                );
            }
            Err(other_error) => panic!("Unexpected error: {other_error}"),
        };

        self.play(ForceRewind::No);
    }

    fn apply_shuffle(&mut self, playlist_name: &str) {
        if playlist_name.ends_with("_shuf") {
            self.client.random(true).unwrap();
        } else {
            let random = self.mode.settings().random;
            self.client.random(random).unwrap();
        }
        self.client.pause().unwrap();
    }

    #[instrument]
    fn switch_playlist(&mut self, direction: Direction) {
        let current_playlist_name =
            match self.db.fetch_playlist_name(&self.mode) {
                Some(playlist_name) => playlist_name,
                None => self.first_playlist_for_mode().unwrap(),
            };
        self.store_position(&current_playlist_name);
        self.save_playlist_if_necessary(&current_playlist_name);
        self.db
            .store_last_played(&current_playlist_name, Db::now_timestamp());

        let new_playlist_name = if let Some(playlist_name) =
            self.playlist_for_mode(direction, &current_playlist_name)
        {
            playlist_name
        } else {
            current_playlist_name
        };

        info!("Switching to playlist {}", new_playlist_name);
        self.load_playlist(&new_playlist_name);
        self.db.store_playlist_name(&self.mode, &new_playlist_name);
        self.apply_shuffle(&new_playlist_name);

        let new_position = self.db.fetch_position(&new_playlist_name);
        self.load_position(new_position);

        //self.play(ForceRewind::Yes);
    }

    pub fn prev_playlist(&mut self) {
        self.switch_playlist(Direction::Previous);
    }

    pub fn next_playlist(&mut self) {
        self.switch_playlist(Direction::Next);
    }

    /// Meditation mode is only enabled at night
    pub fn is_meditation_time() -> bool {
        const START_HOUR: i8 = 22;
        const START_MIN: i8 = 30;
        const END_HOUR: i8 = 9;
        const END_MIN: i8 = 0;

        let now = crate::time::now().time();
        debug!("Checking if it is meditation time: now is {:?}", now);
        let start = Time::new(START_HOUR, START_MIN, 0, 0).unwrap();
        let end = Time::new(END_HOUR, END_MIN, 0, 0).unwrap();

        debug!("Meditation start time: {}, end time: {}", start, end);
        let start_sm_now = start < now;
        let now_sm_end = now < end;
        debug!("start < now: {}, now < end: {}", start_sm_now, now_sm_end);

        start < now || now < end
    }

    fn fetch_current_mode(&self) -> Option<AudioMode> {
        self.db.fetch_mode()
    }

    fn store_current_mode(&self) {
        self.db.store_mode(&self.mode);
    }

    pub fn next_mode(&mut self) {
        let current_playlist_name =
            match self.db.fetch_playlist_name(&self.mode) {
                Some(playlist_name) => playlist_name,
                None => self.first_playlist_for_mode().unwrap(),
            };
        self.store_position(&current_playlist_name);
        self.save_playlist_if_necessary(&current_playlist_name);
        self.db
            .store_last_played(&current_playlist_name, Db::now_timestamp());

        self.mode.next();
        info!("Switching to mode {:?}", self.mode);

        if self.mode == AudioMode::Meditation && !Self::is_meditation_time() {
            self.mode.next();
            trace!("Skipping meditation");
        }

        // Check if a playlist is stored in the db, and still exists
        let new_playlist_name = self.db.fetch_playlist_name(&self.mode);
        let new_playlist_name = match new_playlist_name {
            Some(playlist_name)
                if self.client.playlist_exists(&playlist_name) =>
            {
                playlist_name
            }
            _ => {
                let playlist_name = self.first_playlist_for_mode().unwrap();
                self.db.store_playlist_name(&self.mode, &playlist_name);
                playlist_name
            }
        };
        self.load_playlist(&new_playlist_name);
        self.store_current_mode();

        let new_position = self.db.fetch_position(&new_playlist_name);
        self.load_position(new_position);

        self.apply_settings(&self.mode.settings());
        self.apply_shuffle(&new_playlist_name);
    }

    fn save_playlist_if_necessary(&mut self, playlist_name: &str) {
        if self.mode.settings().save_playlist {
            self.client.pl_remove(playlist_name).unwrap();
            self.client.save(playlist_name).unwrap();
        }
    }

    #[instrument(ret)]
    fn first_playlist_for_mode(&mut self) -> Option<String> {
        let playlists = self.get_playlists();
        for playlist in playlists {
            if playlist.name.starts_with(self.mode.to_prefix()) {
                return Some(playlist.name);
            }
        }
        None
    }

    #[instrument(ret)]
    fn playlist_for_mode(
        &mut self,
        direction: Direction,
        current_playlist_name: &String,
    ) -> Option<String> {
        let playlists = self.get_playlists();
        assert!(!playlists.is_empty());
        let playlist_names = playlists.into_iter().map(|pl| pl.name);
        let mut playlist_names = playlist_names
            .filter(|pl| pl.starts_with(self.mode.to_prefix()))
            .collect::<Vec<_>>();

        playlist_names.sort();

        if let Direction::Previous = direction {
            playlist_names.reverse();
        }
        assert!(!playlist_names.is_empty());
        let mut playlist_names = playlist_names.iter().cycle().peekable();

        while *playlist_names.peek().unwrap() != current_playlist_name {
            playlist_names.next();
        }
        playlist_names.nth(1).map(std::borrow::ToOwned::to_owned)
    }

    fn store_position(&mut self, playlist_name: &str) {
        let pos_in_pl = if let Some(song) = self.client.status().unwrap().song {
            song.pos
        } else {
            0
        };

        let elapsed = if let Some(elapsed) = self.get_elapsed() {
            elapsed.as_secs().try_into().unwrap()
        } else {
            0
        };

        let position = db::Position { pos_in_pl, elapsed };
        self.db.store_position(playlist_name, &position);
    }

    fn load_playlist(&mut self, playlist_name: &str) {
        self.client.clear().unwrap();
        self.client.load(playlist_name, ..).expect("Should exist");
        self.client.pause().unwrap();
    }

    fn load_position(&mut self, position: Option<db::Position>) {
        if let Some(position) = position {
            self.client.queue().unwrap();
            self.seek_to(position.pos_in_pl, position.elapsed);
        } else {
            self.seek_to(0, 0);
        }
        self.client.pause().unwrap();
    }

    fn seek_to(&mut self, pos_in_pl: u32, elapsed: u32) {
        match self.client.seek(pos_in_pl, elapsed) {
            Ok(_) => (),
            Err(Error::Server(server_error)) => {
                assert!(
                    server_error.detail == "Bad song index",
                    "Unexpected ServerError: {server_error}"
                );
            }
            Err(other_error) => panic!("Unexpected error: {other_error}"),
        }
    }

    fn seek_in_cur(&mut self, elapsed: u32) {
        if let Some(song) = self.client.currentsong().unwrap() {
            if let Some(place) = song.place {
                self.seek_to(place.pos, elapsed);
            }
        }
    }

    fn apply_settings(&mut self, audio_settings: &Settings) {
        self.client.repeat(audio_settings.repeat).unwrap();
        self.client.random(audio_settings.random).unwrap();
        self.client.single(audio_settings.single).unwrap();
        self.client.consume(audio_settings.consume).unwrap();
        self.client.volume(audio_settings.volume).unwrap();
        self.client.pause().unwrap();
    }

    pub(crate) fn go_to_mode(&mut self, target_mode: &AudioMode) {
        while self.mode != *target_mode {
            self.next_mode();
        }
    }

    pub(crate) fn go_to_playlist(
        &mut self,
        target_playlist: &str,
    ) -> Result<(), String> {
        let old_playlist = self.db.fetch_playlist_name(&self.mode).unwrap();
        while self.db.fetch_playlist_name(&self.mode).unwrap()
            != *target_playlist
        {
            self.switch_playlist(Direction::Next);
            if self.db.fetch_playlist_name(&self.mode).unwrap() == old_playlist
            {
                return Err(format!(
                    "Could not find target playlist {target_playlist:?} to go to"
                ));
            }
        }

        Ok(())
    }

    pub(crate) async fn go_to_mode_playlist(
        &mut self,
        mode: &AudioMode,
        playlist: &str,
    ) {
        self.go_to_mode(mode);
        tokio::time::sleep(Duration::from_millis(100)).await;
        match self.go_to_playlist(playlist) {
            Ok(()) => (),
            Err(e) => println!("{e}"),
        };
    }

    pub(crate) async fn start_wakeup_music(&mut self) {
        self.reconnect().unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;

        let pl_name = "music_wakeup";
        self.create_wakeup_playlist(pl_name).await;
        self.go_to_mode_playlist(&AudioMode::Music, pl_name).await;

        tokio::time::sleep(Duration::from_millis(100)).await;
        self.load_playlist(pl_name);
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.load_position(None);
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.play(ForceRewind::No);
    }

    async fn create_wakeup_playlist(&mut self, pl_name: &str) {
        let slow_songs = self.client.playlist("slow").unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let normal_songs = self.client.playlist("music_all_shuf").unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        self.client.pl_clear(pl_name).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        let to_add = {
            let mut rng = rand::rng();

            slow_songs
                .choose_multiple(&mut rng, 1)
                .chain(normal_songs.choose_multiple(&mut rng, 30))
        };

        self.client.pl_push(
            pl_name,
            &Song {
                file: "noise.ogg".to_string(),
                ..Default::default()
            },
        ).unwrap();
        for song in to_add {
            self.client.pl_push(pl_name, song).unwrap();
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    pub(crate) fn insert_next(&mut self, song_path: &str) {
        if let Ok(id) = self.client.push(song_path) {
            let _ = self.client.prioid(id, 128);
        };
    }
}
