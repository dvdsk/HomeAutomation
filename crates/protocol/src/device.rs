use core::time::Duration;

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::{large_bedroom, small_bedroom, Affector, Reading};

#[derive(
    Clone,
    Debug,
    defmt::Format,
    Serialize,
    Deserialize,
    MaxSize,
    PartialEq,
    Eq,
    Hash,
)]
pub enum Device {
    LargeBedroom(large_bedroom::Device),
    SmallBedroom(small_bedroom::Device),
}
impl Device {
    #[must_use]
    pub const fn info(&self) -> Info {
        match self {
            Device::LargeBedroom(dev) => dev.info(),
            Device::SmallBedroom(dev) => dev.info(),
        }
    }
}

#[derive(Debug)]
pub struct Info {
    pub name: &'static str,
    pub affects_readings: &'static [Reading],
    pub affectors: &'static [Affector],
    pub min_sample_interval: Duration,
    pub max_sample_interval: Duration,
    pub temporal_resolution: Duration,
}

impl Info {
    pub(crate) const fn button_defaults() -> Self {
        Self {
            name: "Buttons",
            affects_readings: &[],
            temporal_resolution: Duration::from_millis(1),
            min_sample_interval: Duration::from_millis(2),
            max_sample_interval: Duration::MAX,
            affectors: &[],
        }
    }
}
