use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::button_enum;

// No these are not borg, these are buttons on a string of cat5.
// They are numbered starting at the farthest from the end
button_enum! {
    DeskButton {
        OneOfFour,
        TwoOfFour,
        ThreeOfFour,
        FourOfFour,
        OneOfThree,
        TwoOfThree,
        ThirdOfThree,
    }
}

// No these are not borg, these are buttons on a string of cat5.
// They are numbered starting at the farthest from the end
button_enum! {
    BedButton {
        OneOfFour,
        TwoOfFour,
        ThreeOfFour,
        FourOfFour,
        OneOfThree,
        TwoOfThree,
        ThirdOfThree,
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    MaxSize,
)]
pub enum LargeBedroom {
    Bed(Bed),
    Desk(Desk),
}

#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    MaxSize,
)]
pub enum Desk {
    Button(DeskButton),
}

#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    MaxSize,
)]
pub enum Bed {
    /// lux divided by 100
    Brightness(u32),
    /// celsius divided by 100
    Temperature(u16),
    /// relative percentage divided by 100
    Humidity(u16),
    /// parts per million
    Co2(u16),
    Button,
}
