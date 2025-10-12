use core::time::Duration;

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use crate::reading::tree::{Item, ItemMut, Tree};
#[cfg(feature = "alloc")]
use crate::reading::FloatLabelFormatter;
#[cfg(feature = "alloc")]
use crate::reading::Info;
use crate::shared::impl_is_same_as;
#[cfg(feature = "alloc")]
use crate::Unit;

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
#[repr(u8)]
pub enum Reading {
    Temperature(f32) = 0,
    Humidity(f32) = 1,
    Pressure(f32) = 2,
}

#[cfg(feature = "alloc")]
impl Tree for Reading {
    fn inner(&self) -> Item<'_> {
        let info = match self {
            Reading::Temperature(val) => Info {
                val: *val,
                device: Device::Bme280.rooted(),
                resolution: 0.01,
                range: -10.0..=45.0,
                unit: Unit::C,
                description: "Temperature",
                branch_id: self.branch_id(),
                label_formatter: Box::new(FloatLabelFormatter),
            },
            Reading::Humidity(val) => Info {
                val: *val,
                device: Device::Bme280.rooted(),
                resolution: 0.008,
                range: 0.0..=100.0,
                unit: Unit::RH,
                description: "Humidity",
                branch_id: self.branch_id(),
                label_formatter: Box::new(FloatLabelFormatter),
            },
            Reading::Pressure(val) => Info {
                val: *val,
                device: Device::Bme280.rooted(),
                range: 87_000.0..=108_100.0,
                resolution: 0.18,
                unit: Unit::Pa,
                description: "Air pressure",
                branch_id: self.branch_id(),
                label_formatter: Box::new(FloatLabelFormatter),
            },
        };

        Item::Leaf(info)
    }

    fn inner_mut(&mut self) -> ItemMut<'_> {
        use crate::reading::tree::field_as_any;

        let value = field_as_any!(self, Temperature, Humidity, Pressure);
        ItemMut::Leaf(value)
    }

    fn branch_id(&self) -> crate::reading::tree::Id {
        ReadingDiscriminants::from(self) as crate::reading::tree::Id
    }
}

impl_is_same_as!(Reading; Temperature, Humidity, Pressure);

#[derive(
    Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq,
)]
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
            Self::Running(sensor_err) | Self::Setup(sensor_err) => {
                sensor_err.device()
            }
            Self::SetupTimedOut(device) | Self::Timeout(device) => {
                device.clone()
            }
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Running(e) => {
                write!(f, "{} ran into error: {e}", e.device())
            }
            Error::Setup(e) => {
                write!(f, "{} errored during setup: {e}", e.device())
            }
            Error::SetupTimedOut(d) => write!(f, "{d} timed out during setup"),
            Error::Timeout(d) => write!(f, "{d} timed out while running"),
        }
    }
}

#[derive(
    Clone, Debug, defmt::Format, Serialize, Deserialize, Eq, PartialEq,
)]
pub enum SensorError {
    BmeError(heapless::String<200>),
}

impl MaxSize for SensorError {
    const POSTCARD_MAX_SIZE: usize = 200 + 1;
}

impl SensorError {
    #[must_use]
    pub fn device(&self) -> Device {
        match self {
            SensorError::BmeError(_) => Device::Bme280,
        }
    }
}

impl core::fmt::Display for SensorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SensorError::BmeError(e) => write!(f, "{e}"),
        }
    }
}

#[derive(
    Clone,
    Debug,
    defmt::Format,
    Serialize,
    Deserialize,
    MaxSize,
    Eq,
    PartialEq,
    Hash,
)]
pub enum Device {
    Bme280,
}

impl core::fmt::Display for Device {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.info().name)
    }
}

macro_rules! rtree {
    [$($reading:expr),+] => {
        [$(crate::Reading::SmallBedroom(
            crate::small_bedroom::Reading::Desk($reading),
        )),+]
    };
}

impl Device {
    #[must_use]
    pub fn rooted(self) -> crate::Device {
        crate::Device::SmallBedroom(crate::small_bedroom::Device::Desk(self))
    }

    #[must_use]
    pub const fn info(&self) -> crate::DeviceInfo {
        match self {
            Device::Bme280 => crate::DeviceInfo {
                name: "Bme280",
                affects_readings: &rtree![
                    Reading::Temperature(0.0),
                    Reading::Humidity(0.0),
                    Reading::Pressure(0.0)
                ],
                affectors: &[],
                min_sample_interval: Duration::from_secs(5),
                max_sample_interval: Duration::from_secs(5),
                temporal_resolution: Duration::from_secs(1),
            },
        }
    }
}
