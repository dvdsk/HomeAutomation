use core::time::Duration;

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use crate::reading::tree::{Item, Tree};
use crate::IsSameAs;

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
    PlayPause,
    TrackNext,
    TrackPrevious,
    VolumeUp,
    VolumeUpHold,
    VolumeDown,
    VolumeDownHold,
    Dots1InitialPress,
    Dots1ShortRelease,
    Dots1DoublePress,
    Dots1LongPress,
    Dots1LongRelease,
    Dots2InitialPress,
    Dots2ShortRelease,
    Dots2DoublePress,
    Dots2LongPress,
    Dots2LongRelease,
}

#[cfg(feature = "alloc")]
impl Tree for Reading {
    #[must_use]
    fn inner(&self) -> Item<'_> {
        Item::Leaf(crate::reading::Info {
            val: 1.0,
            device: crate::Device::SmallBedroom(
                super::Device::PortableButtonPanel(Device),
            ),
            description: "button",
            range: 0.0..3.0,
            resolution: 1.0,
            unit: crate::Unit::None,
            branch_id: self.branch_id(),
        })
    }

    fn name(&self) -> String {
        let dbg_repr = format!("{self:?}");
        dbg_repr
    }

    #[must_use]
    fn branch_id(&self) -> crate::reading::tree::Id {
        ReadingDiscriminants::from(self) as crate::reading::tree::Id
    }
}

impl IsSameAs for Reading {
    fn is_same_as(&self, other: &Self) -> bool {
        self.eq(other)
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
            crate::small_bedroom::$thing::PortableButtonPanel($item),
        )),+]
    };
}

impl Device {
    /// Note the order in which the `affects_readings` occur is the order in which
    /// they will be stored, do not change it!
    #[must_use]
    pub const fn info(&self) -> crate::DeviceInfo {
        crate::DeviceInfo {
            name: "Ikea Symfonisk",
            affects_readings: &tree![Reading;
                Reading::PlayPause,
                Reading::TrackNext,
                Reading::TrackPrevious,
                Reading::VolumeUp,
                Reading::VolumeUpHold,
                Reading::VolumeDown,
                Reading::VolumeDownHold,
                Reading::Dots1InitialPress,
                Reading::Dots1ShortRelease,
                Reading::Dots1DoublePress,
                Reading::Dots1LongPress,
                Reading::Dots1LongRelease,
                Reading::Dots2InitialPress,
                Reading::Dots2ShortRelease,
                Reading::Dots2DoublePress,
                Reading::Dots2LongPress,
                Reading::Dots2LongRelease
            ],
            temporal_resolution: Duration::from_secs(1), // unknown
            min_sample_interval: Duration::from_secs(5), // unknown
            max_sample_interval: Duration::MAX,
            affectors: &[],
        }
    }
}
