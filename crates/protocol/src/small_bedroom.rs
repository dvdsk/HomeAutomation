use serde::{Deserialize, Serialize};
use postcard::experimental::max_size::MaxSize;

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
}

#[derive(Clone, Copy, Debug, defmt::Format, Serialize, Deserialize, MaxSize)]
pub enum SmallBedroom {
    Desk(ButtonPanel),
}
