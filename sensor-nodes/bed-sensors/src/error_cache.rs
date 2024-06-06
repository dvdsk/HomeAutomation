use core::fmt;

// use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
// use embassy_sync::blocking_mutex::Mutex;
// use fasthash::{murmur3, Murmur3Hasher, SeaHasher};
// use heapless::LinearMap;
use protocol::large_bedroom::bed::{self, Device};
use protocol::make_error_string;

use crate::sensors::{I2cError, UartError};
/// static global error fmt message cache
// use protocol::large_bedroom::bed::Error;

// static ERROR_CACHE: Mutex<CriticalSectionRawMutex, LinearMap<u32, heapless::String<200>, 20>> =
//     Mutex::new(LinearMap::new());

#[derive(PartialEq, Eq)]
pub enum SensorError {
    Mhz14(mhzx::Error<UartError, UartError>),
    Sps30(sps30_async::Error<UartError, UartError>),
    Sht31(sht31::SHTError),
    Bme680(bosch_bme680::BmeError<I2cError>),
    Max44(max44009::Error<I2cError>),
}

impl Into<bed::SensorError> for SensorError {
    fn into(self) -> bed::SensorError {
        match self {
            SensorError::Mhz14(e) => bed::SensorError::Mhz14(make_error_string(e)),
            SensorError::Sps30(e) => bed::SensorError::Sps30(make_error_string(e)),
            SensorError::Sht31(e) => bed::SensorError::Sht31(make_error_string(e)),
            SensorError::Bme680(e) => bed::SensorError::Bme680(make_error_string(e)),
            SensorError::Max44(e) => bed::SensorError::Max44(make_error_string(e)),
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum Error {
    Running(SensorError),
    Setup(SensorError),
    Timeout(Device),
    SetupTimedOut(Device),
}

impl Into<bed::Error> for Error {
    fn into(self) -> bed::Error {
        match self {
            Error::Running(e) => bed::Error::Running(e.into()),
            Error::Setup(e) => bed::Error::Setup(e.into()),
            Error::Timeout(dev) => bed::Error::Timeout(dev),
            Error::SetupTimedOut(dev) => bed::Error::SetupTimedOut(dev),
        }
    }
}
