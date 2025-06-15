use core::time::Duration;

#[cfg(feature = "alloc")]
use crate::affector::{self, Control, ControlValue};
use crate::button::Press;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use crate::reading::tree::{Item, Tree};
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
    Pressure(f32) = 1,
    FanPower(f32) = 2,
    Button(Press) = 3,
}

impl_is_same_as! {Reading; Temperature, Pressure, FanPower, Button}

#[cfg(feature = "alloc")]
impl Tree for Reading {
    fn inner(&self) -> Item<'_> {
        let leaf = match self {
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
            Reading::FanPower(val) => Info {
                val: *val as u8 as f32,
                device: crate::Device::LargeBedroom(super::Device::Airbox(
                    Device::Fans,
                )),
                resolution: 1.0,
                range: 0.0..=100.0,
                unit: Unit::RelativePower,
                description: "Fan power",
                branch_id: self.branch_id(),
                label_formatter: Box::new(FloatLabelFormatter),
            },
            Reading::Button(press) => Info {
                val: if press.is_long() { 2.0 } else { 1.0 },
                device: crate::Device::LargeBedroom(super::Device::Airbox(
                    Device::Gpio,
                )),
                description: "button",
                range: 0.0..=2.0,
                resolution: 1.0,
                unit: crate::Unit::None,
                branch_id: self.branch_id(),
                label_formatter: Box::new(crate::button::ButtonLabelFormatter),
            },
        };
        Item::Leaf(leaf)
    }

    fn branch_id(&self) -> crate::reading::tree::Id {
        ReadingDiscriminants::from(self) as crate::reading::tree::Id
    }
}

#[derive(
    Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq,
)]
pub enum Error {
    Running(DeviceError),
    Setup(DeviceError),
    SetupTimedOut(Device),
    Timeout(Device),
}

impl Error {
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
pub enum DeviceError {
    BmeError(heapless::String<200>),
    Pwm(heapless::String<200>),
}

impl DeviceError {
    fn device(&self) -> Device {
        Device::Bme280
    }
}

impl core::fmt::Display for DeviceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DeviceError::BmeError(e) => write!(f, "Bme error: {e}"),
            DeviceError::Pwm(e) => write!(f, "Pwm error:{e}"),
        }
    }
}

impl MaxSize for DeviceError {
    const POSTCARD_MAX_SIZE: usize = 200 + 1;
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
    Fans,
    Gpio,
}

impl core::fmt::Display for Device {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.info().name)
    }
}

macro_rules! tree {
    [$thing:ident; $($item:expr),+] => {
        [$(crate::$thing::LargeBedroom(
            crate::large_bedroom::$thing::Airbox($item),
        )),+]
    };
}

impl Device {
    #[must_use]
    pub fn rooted(self) -> crate::Device {
        crate::Device::LargeBedroom(crate::large_bedroom::Device::Airbox(self))
    }

    /// Note the order in which the `affects_readings` occur is the order in which
    /// they will be stored, do not change it!
    #[must_use]
    pub const fn info(&self) -> crate::DeviceInfo {
        match self {
            Device::Bme280 => crate::DeviceInfo {
                name: "Bme280",
                affects_readings: &tree![
                Reading; Reading::Temperature(0.0), Reading::Pressure(0.0)],
                temporal_resolution: Duration::from_secs(5), // unknown
                min_sample_interval: Duration::from_secs(5), // unknown
                max_sample_interval: Duration::MAX,
                affectors: &[],
            },
            Device::Fans => crate::DeviceInfo {
                name: "Fans",
                affects_readings: &tree![Reading; Reading::FanPower(0.0) ],
                temporal_resolution: Duration::from_secs(5), // unknown
                min_sample_interval: Duration::from_secs(5), // unknown
                max_sample_interval: Duration::MAX,
                affectors: &[],
            },
            Self::Gpio => crate::DeviceInfo {
                name: "Gpio",
                affects_readings: &tree![Reading; Reading::Button(Press(0)) ],
                temporal_resolution: Duration::from_millis(50), // unknown
                min_sample_interval: Duration::from_millis(50), // unknown
                max_sample_interval: Duration::MAX,
                affectors: &[],
            },
        }
    }
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
    Eq,
    Hash,
)]
#[strum_discriminants(derive(Hash))]
pub enum Affector {
    FanPower { power: u8 },
    ResetNode,
}

impl crate::IsSameAs for Affector {
    fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Affector::FanPower { .. }, Affector::FanPower { .. }) => true,
            (Affector::ResetNode, Affector::ResetNode) => true,
            _ => false,
        }
    }
}

impl Affector {
    #[cfg(feature = "alloc")]
    pub fn controls(&mut self) -> Vec<Control> {
        match self {
            Affector::FanPower { power } => vec![Control {
                name: "red",
                value: ControlValue::SetNum {
                    valid_range: 0..100,
                    value: *power as usize,
                    setter: Some(Box::new(|input: usize| *power = input as u8)),
                },
            }],
            Affector::ResetNode => vec![Control {
                name: "reset the node",
                value: ControlValue::Trigger,
            }],
        }
    }
}

#[cfg(feature = "alloc")]
impl affector::tree::Tree for Affector {
    fn inner(&self) -> affector::tree::Item<'_> {
        let description = match self {
            Affector::FanPower { .. } => "Set the power of the fans",
            Affector::ResetNode => "Reset the node, this might fix errors such as I2c getting stuck after Arbitration Error",
        };

        affector::tree::Item::Leaf(affector::Info { description })
    }

    fn branch_id(&self) -> affector::tree::Id {
        AffectorDiscriminants::from(self) as affector::tree::Id
    }
}
