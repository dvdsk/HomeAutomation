#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Io error")]
    IO(#[from] std::io::Error),
    #[error("Could not parse yaml")]
    YamlParsing(#[from] serde_yaml::Error),
    #[error("Database error")]
    DataBase(#[from] sled::Error),
    #[error("Error while adjusting lighting: {0}")]
    Lamps(#[from] crate::system::lamps::Error),
    #[error("Error contacting mpd")]
    Mpd(#[from] mpd::error::Error),
    #[error("Error in the wakeup alarm system")]
    SetWakeUp(#[from] crate::input::jobs::wakeup::Error),
}
