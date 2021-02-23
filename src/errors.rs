use fern::colors::{Color, ColoredLevelConfig};

use crate::input::web_api::server;
use crate::input::youtube_downloader;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Io error")]
    IO(#[from] std::io::Error),
    #[error("Could not parse yaml")]
    YamlParsing(#[from] serde_yaml::Error),
    #[error("Channel send error")]
    Channel(#[from] crossbeam_channel::SendError<()>),
    #[error("Database error")]
    DataBase(#[from] sled::Error),
    #[error("Could not download youtube")]
    Download(#[from] reqwest::Error),
    #[error("Error contacting hue")]
    Lamps(#[from] philipshue::errors::HueError),
    #[error("Error contacting mpd")]
    Mpd(#[from] mpd::error::Error),
    #[error("TODO")]
    RetryLogic(String),
    #[error("Could not download youtube movie")]
    YoutubeDownloader(youtube_downloader::Error),
    #[error("Problem handeling web requests")]
    WebServerError(server::Error),
    #[error("Error in the wakeup alarm system")]
    SetWakeUp(#[from] crate::input::jobs::wakeup::Error),
    #[error("Error in wakeup state")]
    WakeUpState(#[from] crate::controller::WakeUpStateError),
    #[error("untracked error")]
    UnTracked,
}

pub fn setup_logging(verbosity: u8) -> Result<(), fern::InitError> {
    let mut base_config = fern::Dispatch::new();
    let colors = ColoredLevelConfig::new()
        .info(Color::Green)
        .debug(Color::Yellow)
        .warn(Color::Magenta);

    base_config = match verbosity {
        0 => base_config.level(log::LevelFilter::Error),
        1 => base_config.level(log::LevelFilter::Warn),
        2 => base_config
            .level(log::LevelFilter::Info)
            .level_for("actix-web", log::LevelFilter::Warn),
        3 => base_config.level(log::LevelFilter::Trace),
        4 => base_config.level(log::LevelFilter::Error),
        _4_or_more => base_config.level(log::LevelFilter::Warn),
    };

    // Separate file config so we can include year, month and day in file logs
    let file_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file("program.log")?);

    let stdout_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format("%H:%M"),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .chain(std::io::stdout());

    base_config
        .chain(file_config)
        .chain(stdout_config)
        .apply()?;
    Ok(())
}
