//https://doc.rust-lang.org/std/convert/trait.From.html
use std::fmt;

pub enum Error {
    IO(std::io::Error),
    GPIO(sysfs_gpio::Error),
    YamlParsing(serde_yaml::Error),
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

impl From<sysfs_gpio::Error> for Error {
    fn from(error: sysfs_gpio::Error) -> Self {
        Error::GPIO(error)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(error) => error.fmt(f),
            Error::GPIO(error) => error.fmt(f),
            Error::YamlParsing(error) => error.fmt(f),
        }
    } 
}