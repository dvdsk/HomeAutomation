#![cfg_attr(not(feature = "alloc"), no_std)]

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

pub mod large_bedroom;
pub mod small_bedroom;

pub mod button;
pub(crate) use button::button_enum;

#[cfg(feature = "alloc")]
pub mod tomato;
#[cfg(feature = "alloc")]
pub(crate) use tomato::all_nodes;

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
#[strum_discriminants(derive(Hash))]
pub enum Reading {
    LargeBedroom(large_bedroom::Reading),
    // SmallBedroom(small_bedroom::Reading),
    // Test,
}

impl Reading {
    pub fn version() -> u8 {
        0u8
    }
}

#[cfg(feature = "alloc")]
all_nodes! {Reading; ReadingDiscriminants; LargeBedroom} //, Test}

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

impl Error {
    pub fn affected_readings(&self) -> impl Iterator<Item = Reading> {
        match self {
            Error::LargeBedroom(error) => error.broken_readings().map(Reading::LargeBedroom),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::LargeBedroom(error) => write!(f, "{}", error),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorMessage<const MAX_ITEMS: usize> {
    pub values: heapless::Vec<Reading, MAX_ITEMS>,
    pub version: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub error: Error,
    pub version: u8,
}

#[derive(Debug)]
pub enum Msg<const M: usize> {
    Readings(SensorMessage<M>),
    ErrorReport(ErrorReport),
}

impl<const M: usize> Msg<M> {
    pub const READINGS: u8 = 1;
    pub const ERROR_REPORT: u8 = 2;

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
        error("Got an unknown message type, expected zero or one got: {}")
    )]
    IncorrectMsgType(u8),
}

impl<const MAX_ITEMS: usize> SensorMessage<MAX_ITEMS> {
    /// the 2x is the max overhead from COBS encoding the encoded data
    /// +2 is for the version
    /// +4 covers the length of the heapless list
    pub const ENCODED_SIZE: usize = 2 * (MAX_ITEMS * Reading::POSTCARD_MAX_SIZE + 2 + 4);

    pub fn new() -> Self {
        Self {
            values: heapless::Vec::new(),
            version: 0,
        }
    }

    pub fn space_left(&self) -> bool {
        self.values.len() < self.values.capacity()
    }

    #[cfg(feature = "alloc")]
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec_cobs(self).expect("Encoding should not fail")
    }

    /// Buffer should be at least Self::ENCODED_SIZE long. The returned slice contains
    /// the serialized data. It can be shorter then the input buffer.
    pub fn encode_slice<'a>(&self, buf: &'a mut [u8]) -> &'a mut [u8] {
        postcard::to_slice_cobs(self, buf).expect("Encoding should not fail")
    }

    pub fn decode(mut bytes: impl AsMut<[u8]>) -> Result<Self, DecodeError> {
        postcard::from_bytes_cobs(bytes.as_mut()).map_err(DecodeError::CorruptEncoding)
    }

    pub fn version(&self) -> u8 {
        self.version
    }
}

impl ErrorReport {
    /// the 2x is the max overhead from COBS encoding the encoded data
    /// +2 is for the version
    /// +4 covers the length of the heapless list
    pub const ENCODED_SIZE: usize = 2 * (Error::POSTCARD_MAX_SIZE + 2);

    pub fn new(error: Error) -> Self {
        Self { error, version: 0 }
    }

    #[cfg(feature = "alloc")]
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec_cobs(self).expect("Encoding should not fail")
    }

    /// Buffer should be at least Self::ENCODED_SIZE long. The returned slice contains
    /// the serialized data. It can be shorter then the input buffer.
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
    defmt::info!("making error string");
    use core::fmt::Write;

    let mut s = ErrorString::new();
    core::write!(s, "{e:?}").ok();
    defmt::info!("done");
    s
}
