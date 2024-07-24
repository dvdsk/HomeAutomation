#![cfg_attr(not(feature = "alloc"), no_std)]
#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

use core::fmt::Display;
use core::time::Duration;

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

pub mod large_bedroom;
pub mod small_bedroom;

pub mod button;
pub(crate) use button::button_enum;

#[cfg(feature = "alloc")]
pub mod reading_tree;

#[derive(Debug)]
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

#[derive(
    strum::EnumDiscriminants,
    strum::VariantNames,
    Clone,
    Debug,
    defmt::Format,
    Serialize,
    Deserialize,
    MaxSize,
    PartialEq,
)]
#[strum_discriminants(derive(Hash))]
pub enum Reading {
    LargeBedroom(large_bedroom::Reading),
    SmallBedroom(small_bedroom::Reading),
    // Test,
}

#[cfg(feature = "alloc")]
impl Reading {
    #[must_use]
    pub fn from_same_device(&self) -> &'static [Reading] {
        use reading_tree::Tree;
        self.leaf().from_same_device()
    }
    #[must_use]
    pub fn range(&self) -> core::ops::Range<f32> {
        use reading_tree::Tree;
        self.leaf().range
    }
    #[must_use]
    /// the step between the two closest datapoints that are not the same
    pub fn resolution(&self) -> f32 {
        use reading_tree::Tree;
        self.leaf().resolution
    }
    #[must_use]
    pub fn device(&self) -> Device {
        use reading_tree::Tree;
        self.leaf().device
    }
}
impl Reading {
    #[must_use]
    pub fn version() -> u8 {
        0u8
    }
}

#[cfg(feature = "alloc")]
reading_tree::all_nodes! {Reading; ReadingDiscriminants; LargeBedroom, SmallBedroom}

#[derive(
    strum::EnumDiscriminants,
    strum::VariantNames,
    Clone,
    Debug,
    defmt::Format,
    Serialize,
    Deserialize,
    MaxSize,
)]
pub enum Error {
    LargeBedroom(large_bedroom::Error),
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, PartialEq, Eq, Hash)]
pub enum Device {
    LargeBedroom(large_bedroom::Device),
    SmallBedroom(small_bedroom::Device),
}
impl Device {
    #[must_use]
    pub const fn info(&self) -> DeviceInfo {
        match self {
            Device::LargeBedroom(dev) => dev.info(),
            Device::SmallBedroom(dev) => dev.info(),
        }
    }
}

pub struct DeviceInfo {
    pub name: &'static str,
    pub affects_readings: &'static [Reading],
    pub min_sample_interval: Duration,
    pub temporal_resolution: Duration,
}


impl Error {
    #[must_use]
    pub fn device(&self) -> Device {
        match self {
            Error::LargeBedroom(error) => Device::LargeBedroom(error.device()),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::LargeBedroom(error) => write!(f, "{error}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorMessage<const MAX_ITEMS: usize> {
    pub values: heapless::Vec<Reading, MAX_ITEMS>,
    pub version: u8,
}

#[allow(clippy::large_enum_variant)] // can not use Box on embedded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub error: Error,
    pub version: u8,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // can not use Box on embedded
pub enum Msg<const M: usize> {
    Readings(SensorMessage<M>),
    ErrorReport(ErrorReport),
}

impl<const M: usize> Msg<M> {
    pub const READINGS: u8 = 1;
    pub const ERROR_REPORT: u8 = 2;

    #[must_use]
    pub fn header(&self) -> u8 {
        let header = match self {
            Msg::Readings(_) => Self::READINGS,
            Msg::ErrorReport(_) => Self::ERROR_REPORT,
        };
        assert_ne!(header, 0, "0 is reserved for cobs encoding");
        header
    }

    pub fn decode(mut bytes: impl AsMut<[u8]>) -> Result<Self, DecodeError> {
        let bytes = bytes.as_mut();
        assert!(!bytes.is_empty(), "can not decode nothing (zero bytes)");

        let msg_type = bytes[0];
        let mut bytes = &mut bytes[1..];

        if msg_type == Self::READINGS {
            Ok(Self::Readings(SensorMessage::<M>::decode(&mut bytes)?))
        } else if msg_type == Self::ERROR_REPORT {
            Ok(Self::ErrorReport(ErrorReport::decode(&mut bytes)?))
        } else {
            Err(DecodeError::IncorrectMsgType(msg_type))
        }
    }

    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = match self {
            Msg::Readings(readings) => readings.encode(),
            Msg::ErrorReport(report) => report.encode(),
        };

        bytes.insert(0, self.header());
        bytes
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
pub enum DecodeError {
    #[cfg_attr(feature = "thiserror", error("Could not decode SensorMessage: {0}"))]
    CorruptEncoding(postcard::Error),
    #[cfg_attr(
        feature = "thiserror",
        error("Got an unknown message type, expected zero or one got: {0}")
    )]
    IncorrectMsgType(u8),
}

impl<const MAX_ITEMS: usize> Default for SensorMessage<MAX_ITEMS> {
    fn default() -> Self {
        Self {
            values: heapless::Vec::new(),
            version: 0,
        }
    }
}
impl<const MAX_ITEMS: usize> SensorMessage<MAX_ITEMS> {
    /// the 2x is the max overhead from COBS encoding the encoded data
    /// +2 is for the version
    /// +4 covers the length of the heapless list
    pub const ENCODED_SIZE: usize = 2 * (MAX_ITEMS * Reading::POSTCARD_MAX_SIZE + 2 + 4);

    #[must_use]
    pub fn space_left(&self) -> bool {
        self.values.len() < self.values.capacity()
    }

    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec_cobs(self).expect("Encoding should not fail")
    }

    /// Buffer should be at least `Self::ENCODED_SIZE` long. The returned slice contains
    /// the serialized data. It can be shorter then the input buffer.
    #[must_use]
    pub fn encode_slice<'a>(&self, buf: &'a mut [u8]) -> &'a mut [u8] {
        postcard::to_slice_cobs(self, buf).expect("Encoding should not fail")
    }

    pub fn decode(mut bytes: impl AsMut<[u8]>) -> Result<Self, DecodeError> {
        postcard::from_bytes_cobs(bytes.as_mut()).map_err(DecodeError::CorruptEncoding)
    }

    #[must_use]
    pub fn version(&self) -> u8 {
        self.version
    }
}

impl ErrorReport {
    /// the 2x is the max overhead from COBS encoding the encoded data
    /// +2 is for the version
    /// +4 covers the length of the heapless list
    pub const ENCODED_SIZE: usize = 2 * (Error::POSTCARD_MAX_SIZE + 2);

    #[must_use]
    pub fn new(error: Error) -> Self {
        Self { error, version: 0 }
    }

    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec_cobs(self).expect("Encoding should not fail")
    }

    /// Buffer should be at least `Self::ENCODED_SIZE` long. The returned slice contains
    /// the serialized data. It can be shorter then the input buffer.
    #[must_use]
    pub fn encode_slice<'a>(&self, buf: &'a mut [u8]) -> &'a mut [u8] {
        postcard::to_slice_cobs(self, buf).expect("Encoding should not fail")
    }

    pub fn decode(mut bytes: impl AsMut<[u8]>) -> Result<Self, DecodeError> {
        postcard::from_bytes_cobs(bytes.as_mut()).map_err(DecodeError::CorruptEncoding)
    }
}

pub type ErrorString = heapless::String<200>;
// thiserror does not work on nostd. That will change when this lands:
// https://github.com/rust-lang/rust/issues/103765
// at that point try switching this to fmt::Display
pub fn make_error_string(e: impl core::fmt::Debug) -> ErrorString {
    use core::fmt::Write;

    let mut s = ErrorString::new();
    core::write!(s, "{e:?}").ok();
    s
}
