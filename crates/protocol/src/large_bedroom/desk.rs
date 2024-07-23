#[cfg(feature = "alloc")]
use core::time::Duration;

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::button_enum;
#[cfg(feature = "alloc")]
use crate::reading_tree::{Item, ReadingInfo, Tree};
#[cfg(feature = "alloc")]
use crate::Unit;

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
    crate::Device::LargeBedroom(crate::large_bedroom::Device::Desk(Device::Gpio));
}

#[derive(
    strum::EnumDiscriminants,
    Clone,
    Copy,
    Debug,
    defmt::Format,
    Serialize,
    Deserialize,
    MaxSize,
    PartialEq,
)]
#[strum_discriminants(derive(Hash))]
pub enum Reading {
    Temperature(f32),
    Humidity(f32),
    Pressure(f32),
    Button(Button),
}

#[cfg(feature = "alloc")]
impl Tree for Reading {
    #[must_use]
    fn inner(&self) -> Item<'_> {
        let leaf = match self {
            Reading::Temperature(val) => ReadingInfo {
                val: *val,
                device: Device::Bme280.rooted(),
                resolution: 0.01,
                range: -10.0..45.0,
                unit: Unit::C,
                description: "Temperature",
                branch_id: self.branch_id(),
                min_sample_interval: Duration::from_secs(5),
                temporal_resolution: Duration::from_secs(1),
            },
            Reading::Humidity(val) => ReadingInfo {
                val: *val,
                device: Device::Bme280.rooted(),
                resolution: 0.008,
                range: 0.0..100.0,
                unit: Unit::RH,
                description: "Temperature",
                branch_id: self.branch_id(),
                min_sample_interval: Duration::from_secs(5),
                temporal_resolution: Duration::from_secs(1),
            },
            Reading::Pressure(val) => ReadingInfo {
                val: *val,
                device: Device::Bme280.rooted(),
                range: 87_000.0..108_100.0,
                resolution: 0.18,
                unit: Unit::Pa,
                description: "Air pressure",
                branch_id: self.branch_id(),
                min_sample_interval: Duration::from_secs(5),
                temporal_resolution: Duration::from_secs(1),
            },
            Reading::Button(val) => return Item::Node(val as &dyn Tree),
        };
        Item::Leaf(leaf)
    }

    #[must_use]
    fn branch_id(&self) -> crate::reading_tree::Id {
        ReadingDiscriminants::from(self) as crate::reading_tree::Id
    }
}

impl Reading {
    #[must_use]
    pub fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Reading::Temperature(_), Self::Temperature(_))
            | (Reading::Humidity(_), Self::Humidity(_))
            | (Reading::Pressure(_), Self::Pressure(_)) => true,
            (Reading::Button(a), Self::Button(b)) => a.is_same_as(b),
            _ => false,
        }
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
    #[must_use]
    pub fn device(&self) -> Device {
        match self {
            Self::Running(sensor_err) | Self::Setup(sensor_err) => sensor_err.device(),
            Self::SetupTimedOut(device) | Self::Timeout(device) => device.clone(),
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
    #[must_use]
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
            SensorError::BmeError(e) | SensorError::Gpio(e) => write!(f, "{e}"),
        }
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq, Hash)]
pub enum Device {
    Bme280,
    Gpio,
}

impl core::fmt::Display for Device {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

macro_rules! rtree {
    [$($reading:expr),+] => {
        [$(crate::Reading::LargeBedroom(
            crate::large_bedroom::Reading::Desk($reading),
        )),+]
    };
}

impl Device {
    #[must_use]
    pub fn rooted(self) -> crate::Device {
        crate::Device::LargeBedroom(crate::large_bedroom::Device::Desk(self))
    }
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Device::Bme280 => "Bme280",
            Device::Gpio => "Gpio",
        }
    }
    #[must_use]
    pub fn affected_readings(&self) -> &'static [crate::Reading] {
        match self {
            Device::Bme280 => &rtree![
                Reading::Temperature(0.0),
                Reading::Humidity(0.0),
                Reading::Pressure(0.0)
            ],
            Device::Gpio => todo!(),
        }
    }
}
