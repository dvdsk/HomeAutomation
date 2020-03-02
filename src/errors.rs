use fern::colors::{Color, ColoredLevelConfig};

use crate::input::youtube_downloader;
use crate::input::web_api::server;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    GPIO(gpio_cdev::errors::Error),
    GPIONotFound,
    YamlParsing(serde_yaml::Error),
    Channel(crossbeam_channel::SendError<()>),
    DataBase(sled::Error),
    Download(reqwest::Error),
    Lamps(philipshue::errors::HueError),
    Mpd(mpd::error::Error),
    RetryLogic(String),
    YoutubeDownloader(youtube_downloader::Error),
    WebServerError(server::Error),
    UnTracked,
}

impl From<server::Error> for Error {
    fn from(err: server::Error) -> Self {
        Error::WebServerError(err)
    }
}

impl From<youtube_downloader::Error> for Error {
    fn from(err: youtube_downloader::Error) -> Self {
        Error::YoutubeDownloader(err)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Error::YamlParsing(error)
    }
}
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IO(error)
    }
}

impl From<gpio_cdev::errors::Error> for Error {
    fn from(error: gpio_cdev::errors::Error) -> Self {
        Error::GPIO(error)
    }
}

impl From<crossbeam_channel::SendError<()>> for Error {
    fn from(error: crossbeam_channel::SendError<()>) -> Self {
        Error::Channel(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::Download(error)
    }
}

impl From<retry::Error<Error>> for Error {
    fn from(error: retry::Error<Error>) -> Self {
        match error {
            retry::Error::Operation{error, total_delay: _total_delay, tries: _tries} => error,
            retry::Error::Internal(error_str) => Error::RetryLogic(error_str),
        }
    }
}

impl From<retry::Error<mpd::error::Error>> for Error {
    fn from(error: retry::Error<mpd::error::Error>) -> Self {
        match error {
            retry::Error::Operation{error, total_delay: _total_delay, tries: _tries} => Error::Mpd(error),
            retry::Error::Internal(error_str) => Error::RetryLogic(error_str),
        }
    }
}

impl From<sled::Error> for Error {
    fn from(error: sled::Error) -> Self {
        Error::DataBase(error)
    }
}

impl From<()> for Error {
    fn from(_error: ()) -> Self {
        Error::UnTracked
    }
}

impl From<philipshue::errors::HueError> for Error {
    fn from(error: philipshue::errors::HueError) -> Self {
        Error::Lamps(error)
    }
}

impl From<mpd::error::Error> for Error {
    fn from(error: mpd::error::Error) -> Self {
        Error::Mpd(error)
    }
}

pub fn setup_logging(verbosity: u8) -> Result<(), fern::InitError> {
	let mut base_config = fern::Dispatch::new();
	let colors = ColoredLevelConfig::new()
	             .info(Color::Green)
	             .debug(Color::Yellow)
	             .warn(Color::Magenta);

	base_config = match verbosity {
		0 =>
			base_config
					.level(log::LevelFilter::Error),
		1 =>
			base_config
					.level(log::LevelFilter::Warn),
		2 =>
			base_config.level(log::LevelFilter::Info)
					.level_for("actix-web", log::LevelFilter::Warn),
		3 =>
			base_config.level(log::LevelFilter::Trace),
		4 =>
			base_config.level(log::LevelFilter::Error),
        _4_or_more => 
            base_config.level(log::LevelFilter::Warn),
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

	base_config.chain(file_config).chain(stdout_config).apply()?;
	Ok(())
}