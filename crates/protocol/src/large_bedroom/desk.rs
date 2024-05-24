use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::button_enum;
use crate::downcast_err::{LinuxI2cError, RpiButtonError};

button_enum! {
    /// No these are not borg, these are buttons on a string of cat5.
    /// They are numbered starting at the farthest from the end
    Button {
        OneOfFour,
        TwoOfFour,
        ThreeOfFour,
        FourOfFour,
        OneOfThree,
        TwoOfThree,
        ThreeOfThree,
    }
}

#[derive(
    strum::EnumDiscriminants, Clone, Copy, Debug, defmt::Format, Serialize, Deserialize, MaxSize,
)]
#[strum_discriminants(derive(Hash))]
pub enum Reading {
    Temperature(f32),
    Humidity(f32),
    Pressure(f32),
    Button(Button),
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
pub enum Error {
    Running(SensorError),
    Setup(SetupError),
    SetupTimedOut(Device),
    Timeout(Device),
}

impl Error {
    pub fn broken_readings(&self) -> &[ReadingDiscriminants] {
        match self {
            Self::Running(sensor_err) => sensor_err.broken_readings(),
            Self::Setup(sensor_err) => sensor_err.broken_readings(),
            Self::SetupTimedOut(device) | Self::Timeout(device) => device.broken_readings(),
        }
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
pub enum SetupError {
    BmeError(bme280::Error<LinuxI2cError>),
    Gpio(RpiButtonError),
    I2c(LinuxI2cError),
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
pub enum SensorError {
    BmeError(bme280::Error<LinuxI2cError>),
    Gpio(RpiButtonError),
}

impl SensorError {
    pub fn broken_readings(&self) -> &'static [ReadingDiscriminants] {
        let device = match self {
            SensorError::BmeError(_) => Device::Bme280,
            SensorError::Gpio(_) => Device::Gpio,
        };

        device.broken_readings()
    }
}

impl SetupError {
    pub fn broken_readings(&self) -> &'static [ReadingDiscriminants] {
        let device = match self {
            SetupError::BmeError(_) | SetupError::I2c(_) => Device::Bme280,
            SetupError::Gpio(_) => Device::Gpio,
        };

        device.broken_readings()
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
pub enum Device {
    Bme280,
    Gpio,
}

impl Device {
    pub fn broken_readings(&self) -> &'static [ReadingDiscriminants] {
        match self {
            Device::Bme280 => &[
                ReadingDiscriminants::Temperature,
                ReadingDiscriminants::Humidity,
            ],
            Device::Gpio => todo!(),
        }
    }
}
