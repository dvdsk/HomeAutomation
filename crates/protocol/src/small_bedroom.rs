use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::button::Press;
#[cfg(feature = "alloc")]
use crate::reading_tree::{Item, Tree};

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
pub enum Reading {
    ButtonPanel(ButtonPanel),
}

#[cfg(feature = "alloc")]
impl Tree for Reading {
    fn inner(&self) -> Item<'_> {
        match self {
            Reading::ButtonPanel(inner) => return Item::Node(inner),
        }
    }

    fn branch_id(&self) -> crate::reading_tree::Id {
        ReadingDiscriminants::from(self) as crate::reading_tree::Id
    }
}

impl Reading {
    pub fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ButtonPanel(a), Self::ButtonPanel(b)) => a.is_same_as(b)
        }
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Hash, PartialEq, Eq)]
pub enum Device {
    Gpio,
}

macro_rules! tree {
    [$($button:expr),+] => {
        [$(crate::Reading::SmallBedroom(
            crate::small_bedroom::Reading::ButtonPanel($button),
        )),+]
    };
}

impl Device {
    pub(crate) fn affected_readings(&self) -> &'static [crate::Reading] {
        match self {
            Device::Gpio => &tree![
                ButtonPanel::TopLeft(Press(0)),
                ButtonPanel::TopMiddle(Press(0)),
                ButtonPanel::TopRight(Press(0)),
                ButtonPanel::BottomLeft(Press(0)),
                ButtonPanel::BottomMiddle(Press(0)),
                ButtonPanel::BOttomRight(Press(0))
            ]
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Device::Gpio => "Gpio",
        }
    }
}
