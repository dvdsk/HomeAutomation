use core::time::Duration;

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::button::Press;
#[cfg(feature = "alloc")]
use crate::affector;
#[cfg(feature = "alloc")]

pub mod desk;
pub mod bed;

crate::button_enum! {
    /// +-----------------------------------------+
    /// | TopLeft    | TopMiddle    | TopRight    |
    /// +-----------------------------------------+
    /// | BottomLeft | BottomMiddle | BottomRight |
    /// +-----------------------------------------+
    ButtonPanel {
        TopLeft,
        TopMiddle,
        TopRight,
        BottomLeft,
        BottomMiddle,
        BOttomRight,
    }
    crate::Device::SmallBedroom(crate::small_bedroom::Device::Gpio);
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
#[repr(u8)]
pub enum Reading {
    ButtonPanel(ButtonPanel) = 0,
    Desk(desk::Reading) = 1,
    Bed(bed::Reading) = 2,
}

crate::reading::tree::all_nodes!{Reading; ReadingDiscriminants; ButtonPanel, Desk, Bed}

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
    Desk(desk::Error),
    Bed(bed::Error),
}

impl Error {
    #[must_use]
    pub fn device(&self) -> Device {
        match self {
            Error::Desk(error) => Device::Desk(error.device()),
            Error::Bed(error) => Device::Bed(error.device()),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Desk(error) => write!(f, "{error}"),
            Error::Bed(error) => write!(f, "{error}"),
        }
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Hash, PartialEq, Eq)]
pub enum Device {
    Gpio,
    Desk(desk::Device),
    Bed(bed::Device),
}

macro_rules! tree {
    [$($button:expr),+] => {
        [$(crate::Reading::SmallBedroom(
            crate::small_bedroom::Reading::ButtonPanel($button),
        )),+]
    };
}

impl Device {
    /// Note the order in which the `affects_readings` occur is the order in which
    /// they will be stored, do not change it!
    #[must_use]
    pub const fn info(&self) -> crate::DeviceInfo {
        match self {
            Device::Gpio => crate::DeviceInfo {
                name: "Gpio",
                affects_readings: &tree![
                    ButtonPanel::TopLeft(Press(0)),
                    ButtonPanel::TopMiddle(Press(0)),
                    ButtonPanel::TopRight(Press(0)),
                    ButtonPanel::BottomLeft(Press(0)),
                    ButtonPanel::BottomMiddle(Press(0)),
                    ButtonPanel::BOttomRight(Press(0))
                ],
                affectors: &[],
                temporal_resolution: Duration::from_millis(1),
                min_sample_interval: Duration::from_millis(2),
                max_sample_interval: Duration::MAX,
            },
            Device::Desk(device) => device.info(),
            Device::Bed(device) => device.info(),
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
