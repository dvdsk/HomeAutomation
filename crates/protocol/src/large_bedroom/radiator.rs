use core::time::Duration;

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use crate::reading::tree::{Item, ItemMut, Tree};
#[cfg(feature = "alloc")]
use crate::reading::FloatLabelFormatter;
#[cfg(feature = "alloc")]
use crate::reading::Info;
use crate::shared::{self, impl_is_same_as};
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
    SetBy(shared::radiator::Source) = 2,
    Setpoint(f32) = 3,
}

impl_is_same_as!{Reading; Temperature, Heating, SetBy, Setpoint}

#[cfg(feature = "alloc")]
impl Tree for Reading {
    fn inner(&self) -> Item<'_> {
        let info = match self {
            Reading::Temperature(val) => Info {
                val: *val,
                device: crate::Device::LargeBedroom(super::Device::Radiator(
                    Device,
                )),
                resolution: 0.01,
                range: -10.0..=45.0,
                unit: Unit::C,
                description: "Temperature",
                branch_id: self.branch_id(),
                label_formatter: Box::new(FloatLabelFormatter),
            },
            Reading::Heating(val) => Info {
                val: *val,
                device: crate::Device::LargeBedroom(super::Device::Radiator(
                    Device,
                )),
                resolution: 1.0,
                range: 0.0..=100.0,
                unit: Unit::RelativePower,
                description: "Heating valve",
                branch_id: self.branch_id(),
                label_formatter: Box::new(FloatLabelFormatter),
            },
            Reading::SetBy(val) => Info {
                val: *val as u8 as f32,
                device: crate::Device::LargeBedroom(super::Device::Radiator(
                    Device,
                )),
                resolution: 1.0,
                range: 0.0..=3.0,
                unit: Unit::None,
                description: "Manual value set",
                branch_id: self.branch_id(),
                label_formatter: Box::new(
                    shared::radiator::SetByLabelFormatter,
                ),
            },
            Reading::Setpoint(val) => Info {
                val: *val,
                device: crate::Device::LargeBedroom(super::Device::Radiator(
                    Device,
                )),
                resolution: 0.5,
                range: 0.0..=30.0,
                unit: Unit::C,
                description: "Set point",
                branch_id: self.branch_id(),
                label_formatter: Box::new(FloatLabelFormatter),
            },
        };

        Item::Leaf(info)
    }
    fn inner_mut(&mut self) -> ItemMut<'_> {
        use crate::reading::tree::field_as_any;

        let value = field_as_any!(self, Temperature, Heating, SetBy, Setpoint);
        ItemMut::Leaf(value)
    }

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
        [$(crate::$thing::LargeBedroom(
            crate::large_bedroom::$thing::Radiator($item),
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
            affects_readings: &tree![
                Reading; Reading::Temperature(0.0), Reading::Heating(0.0), 
                Reading::SetBy(shared::radiator::Source::Manual), 
                Reading::Setpoint(0.0)],
            temporal_resolution: Duration::from_secs(5), // unknown
            min_sample_interval: Duration::from_secs(5), // unknown
            max_sample_interval: Duration::MAX,
            affectors: &[],
        }
    }
}
