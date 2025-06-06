use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use crate::Device;
#[cfg(feature = "alloc")]
use crate::Unit;
use crate::{large_bedroom, small_bedroom};

#[cfg(feature = "alloc")]
pub mod tree;

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
#[repr(u8)]
pub enum Reading {
    LargeBedroom(large_bedroom::Reading) = 0,
    SmallBedroom(small_bedroom::Reading) = 1,
    // Test,
}

#[cfg(feature = "alloc")]
impl Reading {
    #[must_use]
    pub fn from_same_device(&self) -> &'static [Reading] {
        use tree::Tree;
        self.leaf().from_same_device()
    }
    #[must_use]
    pub fn range(&self) -> core::ops::RangeInclusive<f32> {
        use tree::Tree;
        self.leaf().range
    }
    #[must_use]
    /// the step between the two closest data-points that are not the same
    pub fn resolution(&self) -> f32 {
        use tree::Tree;
        self.leaf().resolution
    }
    #[must_use]
    pub fn device(&self) -> Device {
        use tree::Tree;
        self.leaf().device
    }
}
impl Reading {
    #[must_use]
    pub fn version() -> u8 {
        0u8
    }
}

// todo make axis fixed for some info's?
pub enum LabelPositions {
    Flexible,
    Fixed(&'static [f64]),
}

#[cfg(feature = "alloc")]
pub trait LabelFormatter: core::fmt::Debug {
    fn format(&self, value: f64, info: &Info) -> String;
    fn box_clone(&self) -> Box<dyn LabelFormatter>;
    fn positions(&self) -> LabelPositions {
        LabelPositions::Flexible
    }
}

#[cfg(feature = "alloc")]
impl Clone for Box<dyn LabelFormatter> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct FloatLabelFormatter;

#[cfg(feature = "alloc")]
impl LabelFormatter for FloatLabelFormatter {
    fn format(&self, value: f64, info: &Info) -> String {
        format!("{0:.1$}", value, info.precision())
    }
    fn box_clone(&self) -> Box<dyn LabelFormatter> {
        Box::new(Self)
    }
}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub struct Info {
    pub val: f32,
    pub device: Device,
    /// smallest step size the data can make
    pub resolution: f32,
    pub range: core::ops::RangeInclusive<f32>,
    pub unit: Unit,
    pub description: &'static str,
    pub branch_id: u8,
    pub label_formatter: Box<dyn LabelFormatter>,
}

#[cfg(feature = "alloc")]
impl Info {
    #[must_use]
    pub fn from_same_device(&self) -> &'static [Reading] {
        self.device.info().affects_readings
    }

    /// useful for printing/formatting floats
    /// # Example
    /// ```rust
    /// use protocol::Reading;
    /// use protocol::large_bedroom;
    /// use protocol::large_bedroom::desk;
    /// use protocol::reading::tree::Tree;
    ///
    /// let reading =
    /// Reading::LargeBedroom(large_bedroom::Reading::Desk(desk::Reading::Temperature(22.428124)));
    ///
    /// let info = reading.leaf();
    /// let printed = format!("{0:.1$}", info.val, info.precision());
    /// assert_eq!(printed, "22.43");
    /// ```
    #[must_use]
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn precision(&self) -> usize {
        if self.resolution > 1.0 {
            0
        } else {
            self.resolution.log10().abs() as usize
        }
    }
}

#[cfg(feature = "alloc")]
tree::all_nodes! {Reading; ReadingDiscriminants; LargeBedroom, SmallBedroom}
