use core::time::Duration;

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use crate::affector;
use crate::button::Press;

pub mod bed;
pub mod desk;
pub mod portable_button_panel;
pub mod radiator;

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
        BottomRight,
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
    PortableButtonPanel(portable_button_panel::Reading) = 4,
    ButtonPanel(ButtonPanel) = 0,
    Desk(desk::Reading) = 1,
    Bed(bed::Reading) = 2,
    Radiator(radiator::Reading) = 3,
}

#[cfg(feature = "alloc")]
crate::reading::tree::all_nodes! {Reading; ReadingDiscriminants; ButtonPanel, Desk, Bed, Radiator, PortableButtonPanel}

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

#[derive(
    Clone,
    Debug,
    defmt::Format,
    Serialize,
    Deserialize,
    MaxSize,
    Hash,
    PartialEq,
    Eq,
)]
pub enum Device {
    Gpio,
    Desk(desk::Device),
    Bed(bed::Device),
    Radiator(radiator::Device),
    PortableButtonPanel(portable_button_panel::Device),
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
                    ButtonPanel::BottomRight(Press(0))
                ],
                affectors: &[],
                temporal_resolution: Duration::from_millis(1),
                min_sample_interval: Duration::from_millis(2),
                max_sample_interval: Duration::MAX,
            },
            Device::Desk(device) => device.info(),
            Device::Bed(device) => device.info(),
            Device::Radiator(device) => device.info(),
            Device::PortableButtonPanel(device) => device.info(),
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
    pub(crate) fn controls(&mut self) -> Vec<affector::Control<'_>> {
        match self {
            Affector::Bed(a) => a.controls(),
        }
    }
}

#[cfg(feature = "alloc")]
affector::tree::all_nodes! {Affector; AffectorDiscriminants; Bed}
