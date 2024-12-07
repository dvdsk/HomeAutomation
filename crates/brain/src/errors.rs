#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Io error")]
    IO(#[from] std::io::Error),
    #[error("Could not parse yaml")]
    YamlParsing(#[from] serde_yaml::Error),
    #[error("Database error")]
    DataBase(#[from] sled::Error),
    #[error("Error contacting mpd")]
    Mpd(#[from] mpd::error::Error),
}
