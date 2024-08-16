#![cfg_attr(not(feature = "alloc"), no_std)]
#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

use core::fmt::Display;

#[cfg(feature = "alloc")]
pub mod reading_tree;
pub mod button;
pub(crate) use button::button_enum;

pub mod large_bedroom;
pub mod small_bedroom;

mod reading;
mod device;
mod error;
pub mod affector;

mod msg;
pub use msg::{Msg, DecodeMsgError};
pub use msg::sensor::SensorMessage;
pub use msg::error::ErrorReport;
pub use msg::error::make_error_string;
pub use reading::Reading;
pub use device::{Device, DeviceInfo};
pub use error::Error;
pub use affector::Affector;


#[derive(Debug, Clone)]
pub enum Unit {
    Pa,
    C,
    RH,
    Lux,
    Ohm,
    Ppm,
    MicrogramPerM3,
    NumberPerCm3,
    NanoMeter,
    None, // for buttons
}

impl Display for Unit {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Unit::Pa => f.write_str("Pa"),
            Unit::C => f.write_str("°C"),
            Unit::RH => f.write_str("%RH"),
            Unit::Lux => f.write_str("lx"),
            Unit::Ohm => f.write_str("Ω"),
            Unit::Ppm => f.write_str("ppm"),
            Unit::MicrogramPerM3 => f.write_str("µg/m³"),
            Unit::NumberPerCm3 => f.write_str("#/cm³"),
            Unit::NanoMeter => f.write_str("nm"),
            Unit::None => f.write_str(""),
        }
    }
}
