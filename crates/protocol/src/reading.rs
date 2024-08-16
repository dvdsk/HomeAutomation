use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::{large_bedroom, small_bedroom};
#[cfg(feature = "alloc")]
use crate::Device;

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
        use crate::reading_tree::Tree;
        self.leaf().from_same_device()
    }
    #[must_use]
    pub fn range(&self) -> core::ops::Range<f32> {
        use crate::reading_tree::Tree;
        self.leaf().range
    }
    #[must_use]
    /// the step between the two closest datapoints that are not the same
    pub fn resolution(&self) -> f32 {
        use crate::reading_tree::Tree;
        self.leaf().resolution
    }
    #[must_use]
    pub fn device(&self) -> Device {
        use crate::reading_tree::Tree;
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
crate::reading_tree::all_nodes! {Reading; ReadingDiscriminants; LargeBedroom, SmallBedroom}
