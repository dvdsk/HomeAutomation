//use std::fmt;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    GPIO(sysfs_gpio::Error),
    YamlParsing(serde_yaml::Error),
    Channel(crossbeam_channel::SendError<()>),
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

impl From<crossbeam_channel::SendError<()>> for Error {
    fn from(error: crossbeam_channel::SendError<()>) -> Self {
        Error::Channel(error)
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