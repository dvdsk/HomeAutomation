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
//impl From<sled::IVec> for Error {
//    fn from(error: crossbeam_channel::SendError<()>) -> Self {
//        Error::Channel(error)
//    }
//}

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

/*impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        dbg!(("heee"));
        match self {
            Error::IO(error) => error.fmt(f),
            Error::GPIO(error) => error.fmt(f),
            Error::YamlParsing(error) => error.fmt(f),
        }
    } 
}*/