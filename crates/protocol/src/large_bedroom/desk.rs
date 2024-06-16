use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::button_enum;
#[cfg(feature = "alloc")]
use crate::tomato::{Tomato, TomatoItem, TomatoLeaf};

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

#[cfg(feature = "alloc")]
impl Tomato for Reading {
    fn inner<'a>(&'a self) -> TomatoItem<'a> {
        let (val, device) = match self {
            Reading::Temperature(val) => (*val, Device::Bme280),
            Reading::Humidity(val) => (*val, Device::Bme280),
            Reading::Pressure(val) => (*val, Device::Bme280),
            Reading::Button(val) => return TomatoItem::Node(val),
        };
        TomatoItem::Leaf(TomatoLeaf {
            val,
            device: device.as_str(),
            from_same_device: &[], // TODO
        })
    }

    fn id(&self) -> crate::tomato::TomatoId {
        ReadingDiscriminants::from(self) as crate::tomato::TomatoId
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
pub enum Error {
    Running(SensorError),
    Setup(SensorError),
    SetupTimedOut(Device),
    Timeout(Device),
}

impl Error {
    pub fn affected_readings(&self) -> &'static [Reading] {
        match self {
            Self::Running(sensor_err) => sensor_err.device().affected_readings(),
            Self::Setup(sensor_err) => sensor_err.device().affected_readings(),
            Self::SetupTimedOut(device) | Self::Timeout(device) => device.affected_readings(),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Running(e) => write!(f, "{} ran into error: {e}", e.device()),
            Error::Setup(e) => write!(f, "{} errored during setup: {e}", e.device()),
            Error::SetupTimedOut(d) => write!(f, "{d} timed out during setup"),
            Error::Timeout(d) => write!(f, "{d} timed out while running"),
        }
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, Eq, PartialEq)]
pub enum SensorError {
    BmeError(heapless::String<200>),
    Gpio(heapless::String<200>),
}

impl MaxSize for SensorError {
    const POSTCARD_MAX_SIZE: usize = 200 + 1;
}

impl SensorError {
    pub fn device(&self) -> Device {
        match self {
            SensorError::BmeError(_) => Device::Bme280,
            SensorError::Gpio(_) => Device::Gpio,
        }
    }
}

impl core::fmt::Display for SensorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SensorError::BmeError(e) => write!(f, "{e}"),
            SensorError::Gpio(e) => write!(f, "{e}"),
        }
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
pub enum Device {
    Bme280,
    Gpio,
}

impl core::fmt::Display for Device {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Device {
    pub fn as_str(&self) -> &'static str {
        match self {
            Device::Bme280 => "Bme280",
            Device::Gpio => "Gpio",
        }
    }
    pub fn affected_readings(&self) -> &'static [Reading] {
        match self {
            Device::Bme280 => &[Reading::Temperature(0.0), Reading::Humidity(0.0)],
            Device::Gpio => todo!(),
        }
    }
}
