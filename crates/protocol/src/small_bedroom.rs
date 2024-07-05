use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::button::Press;
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
    strum::EnumDiscriminants, Clone, Copy, Debug, defmt::Format, Serialize, Deserialize, MaxSize, PartialEq,
)]
pub enum Reading {
    ButtonPanel(ButtonPanel),
}

impl Tree for Reading {
    fn inner<'a>(&'a self) -> Item<'a> {
        match self {
            Reading::ButtonPanel(inner) => return Item::Node(inner),
        }
    }

    fn id(&self) -> crate::reading_tree::Id {
        ReadingDiscriminants::from(self) as crate::reading_tree::Id
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
            ],
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Device::Gpio => "Gpio",
        }
    }
}
