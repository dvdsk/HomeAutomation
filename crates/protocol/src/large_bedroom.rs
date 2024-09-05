use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

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
#[repr(u8)]
pub enum Reading {
    Bed(bed::Reading) = 0,
    Desk(desk::Reading) = 1,
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
impl Affector {
    #[cfg(feature = "alloc")]
    pub(crate) fn controls(&mut self) -> Vec<affector::Control> {
        match self {
            Affector::Bed(a) => a.controls(),
        }
    }
}

#[cfg(feature = "alloc")]
affector::tree::all_nodes! {Affector; AffectorDiscriminants; Bed}
