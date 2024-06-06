use core::fmt;

// use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
// use embassy_sync::blocking_mutex::Mutex;
// use fasthash::{murmur3, Murmur3Hasher, SeaHasher};
// use heapless::LinearMap;
use protocol::large_bedroom::bed::{self, Device};
use protocol::make_error_string;
/// static global error fmt message cache
// use protocol::large_bedroom::bed::Error;

// static ERROR_CACHE: Mutex<CriticalSectionRawMutex, LinearMap<u32, heapless::String<200>, 20>> =
//     Mutex::new(LinearMap::new());

#[derive(PartialEq, Eq, Clone)]
pub enum SensorError<TxError, RxError, I2cError>
where
    TxError: defmt::Format + fmt::Debug,
    RxError: defmt::Format + fmt::Debug,
    I2cError: defmt::Format + fmt::Debug,
{
    Mhz14(mhzx::Error<TxError, RxError>),
    Sps30(sps30_async::Error<TxError, RxError>),
    Sht31(sht31::SHTError),
    Bme680(bosch_bme680::BmeError<I2cError>),
    Max44(max44009::Error<I2cError>),
}

impl<TxError, RxError, I2cError> Into<bed::SensorError> for SensorError<TxError, RxError, I2cError>
where
    TxError: defmt::Format + fmt::Debug,
    RxError: defmt::Format + fmt::Debug,
    I2cError: defmt::Format + fmt::Debug,
{
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

#[derive(PartialEq, Eq, Clone)]
pub enum Error<TxError, RxError, I2cError>
where
    TxError: defmt::Format + fmt::Debug,
    RxError: defmt::Format + fmt::Debug,
    I2cError: defmt::Format + fmt::Debug,
{
    Running(SensorError<TxError, RxError, I2cError>),
    Setup(SensorError<TxError, RxError, I2cError>),
    Timeout(Device),
    SetupTimedOut(Device),
}

impl<TxError, RxError, I2cError> Into<bed::Error> for Error<TxError, RxError, I2cError>
where
    TxError: defmt::Format + fmt::Debug,
    RxError: defmt::Format + fmt::Debug,
    I2cError: defmt::Format + fmt::Debug,
{
    fn into(self) -> bed::Error {
        match self {
            Error::Running(e) => bed::Error::Running(e.into()),
            Error::Setup(e) => bed::Error::Setup(e.into()),
            Error::Timeout(dev) => bed::Error::Timeout(dev),
            Error::SetupTimedOut(dev) => bed::Error::SetupTimedOut(dev),
        }
    }
}
