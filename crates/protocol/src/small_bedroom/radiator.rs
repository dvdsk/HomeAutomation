use core::time::Duration;

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use crate::reading::tree::{Item, Tree};
#[cfg(feature = "alloc")]
use crate::reading::Info;
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
    Heating(f32) = 1,
}

impl crate::IsSameAs for Reading {
    fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Reading::Temperature(_), Reading::Temperature(_))
            | (Reading::Heating(_), Reading::Heating(_)) => true,
            (_, _) => false,
        }
    }
}

#[cfg(feature = "alloc")]
impl Tree for Reading {
    #[must_use]
    fn inner(&self) -> Item<'_> {
        let leaf = match self {
            Reading::Temperature(val) => Info {
                val: *val,
                device: crate::Device::SmallBedroom(super::Device::Radiator(
                    Device,
                )),
                resolution: 0.01,
                range: -10.0..45.0,
                unit: Unit::C,
                description: "Temperature",
                branch_id: self.branch_id(),
            },
            Reading::Heating(val) => Info {
                val: *val,
                device: crate::Device::SmallBedroom(super::Device::Radiator(
                    Device,
                )),
                resolution: 1.0,
                range: 0.0..100.0,
                unit: Unit::PercentageOpen,
                description: "Heating valve",
                branch_id: self.branch_id(),
            },
        };
        Item::Leaf(leaf)
    }

    #[must_use]
    fn branch_id(&self) -> crate::reading::tree::Id {
        ReadingDiscriminants::from(self) as crate::reading::tree::Id
    }
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
pub struct Device;

impl core::fmt::Display for Device {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.info().name)
    }
}

macro_rules! tree {
    [$thing:ident; $($item:expr),+] => {
        [$(crate::$thing::SmallBedroom(
            crate::small_bedroom::$thing::Radiator($item),
        )),+]
    };
}

impl Device {
    /// Note the order in which the `affects_readings` occur is the order in which
    /// they will be stored, do not change it!
    #[must_use]
    pub const fn info(&self) -> crate::DeviceInfo {
        crate::DeviceInfo {
            name: "Radiator",
            affects_readings: &tree![Reading; Reading::Temperature(0.0), Reading::Heating(0.0)],
            temporal_resolution: Duration::MAX,
            min_sample_interval: Duration::ZERO,
            max_sample_interval: Duration::MAX,
            affectors: &[],
        }
    }
}
