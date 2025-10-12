#![cfg_attr(not(feature = "alloc"), no_std)]
#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

use core::fmt::Display;

pub mod button;
pub(crate) use button::button_enum;

pub mod large_bedroom;
pub mod small_bedroom;
pub mod shared;

pub mod affector;
mod device;
mod error;
pub mod pir;
pub mod reading;
pub mod usb;

mod msg;
pub use affector::Affector;
pub use device::Device;
pub use device::Info as DeviceInfo;
pub use error::Error;
pub use msg::error::{make_error_string, ErrorReport, ErrorString};
pub use msg::sensor::SensorMessage;
pub use msg::{DecodeMsgError, Msg};
pub use reading::Reading;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Unit {
    C,
    Lux,
    MicrogramPerM3,
    NanoMeter,
    None, // for buttons & pir
    NumberPerCm3,
    Ohm,
    Pa,
    RelativePower,
    Ppm,
    RH,
}

impl Display for Unit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Unit::C => f.write_str("°C"),
            Unit::Lux => f.write_str("lx"),
            Unit::MicrogramPerM3 => f.write_str("µg/m³"),
            Unit::NanoMeter => f.write_str("nm"),
            Unit::None => f.write_str(""),
            Unit::NumberPerCm3 => f.write_str("#/cm³"),
            Unit::Ohm => f.write_str("Ω"),
            Unit::Pa => f.write_str("Pa"),
            Unit::RelativePower => f.write_str("% power"),
            Unit::Ppm => f.write_str("ppm"),
            Unit::RH => f.write_str("%RH"),
        }
    }
}

/// Is this the same variant as the other?
/// Example:
/// Temperature(5) is the same as Temperature(6)
pub trait IsSameAs {
    /// Is this the same variant as the other?
    #[must_use]
    fn is_same_as(&self, other: &Self) -> bool;
}
