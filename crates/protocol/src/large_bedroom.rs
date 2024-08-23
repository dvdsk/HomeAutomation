use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::msg::DecodeMsgError;
#[cfg(feature = "alloc")]
use crate::{reading, affector};

pub mod bed;
pub mod desk;

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
    Bed(bed::Reading),
    Desk(desk::Reading),
}

#[cfg(feature = "alloc")]
reading::tree::all_nodes! {Reading; ReadingDiscriminants; Bed, Desk}

#[derive(
    strum::EnumDiscriminants,
    Clone,
    Debug,
    defmt::Format,
    Serialize,
    Deserialize,
    MaxSize,
    Eq,
    PartialEq,
)]
pub enum Error {
    Bed(bed::Error),
    Desk(desk::Error),
}

impl Error {
    #[must_use]
    pub fn device(&self) -> Device {
        match self {
            Error::Bed(error) => Device::Bed(error.device()),
            Error::Desk(error) => Device::Desk(error.device()),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Bed(error) => write!(f, "{error}"),
            Error::Desk(error) => write!(f, "{error}"),
        }
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq, Hash)]
pub enum Device {
    Bed(bed::Device),
    Desk(desk::Device),
}

impl Device {
    #[must_use]
    pub const fn info(&self) -> crate::DeviceInfo {
        match self {
            Self::Bed(dev) => dev.info(),
            Self::Desk(dev) => dev.info(),
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
    Bed(bed::Affector),
}

#[cfg(feature = "alloc")]
affector::tree::all_nodes! {Affector; AffectorDiscriminants; Bed}

impl Affector {
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

    pub fn decode(mut bytes: impl AsMut<[u8]>) -> Result<Self, DecodeMsgError> {
        postcard::from_bytes_cobs(bytes.as_mut()).map_err(DecodeMsgError::CorruptEncoding)
    }

    #[must_use]
    pub fn version(&self) -> u8 {
        0
    }
}
