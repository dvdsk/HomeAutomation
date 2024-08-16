use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::{large_bedroom, Device};

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
    Eq,
)]
pub enum Error {
    LargeBedroom(large_bedroom::Error),
}

impl Error {
    #[must_use]
    pub fn device(&self) -> Device {
        match self {
            Error::LargeBedroom(error) => Device::LargeBedroom(error.device()),
        }
    }

    #[must_use]
    pub const fn max_size() -> usize {
        Self::POSTCARD_MAX_SIZE
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::LargeBedroom(error) => write!(f, "{error}"),
        }
    }
}
