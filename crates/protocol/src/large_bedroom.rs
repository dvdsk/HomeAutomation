use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::button_enum;
use crate::extended_errors::I2cError;

button_enum! {
    /// No these are not borg, these are buttons on a string of cat5.
    /// They are numbered starting at the farthest from the end
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

button_enum! {
    /// All button are on the headboard of the bed. As seen from the foot of the
    /// bed it looks like this:
    ///
    /// LR----------------------------|
    /// |--------------|--------------|
    /// |-----------321|123-----------|
    /// |654-----------|-----------456|
    ///
    /// Legend:
    /// L: TopLeft, R: TopRight, 1: MiddleInner, 2: MiddleCenter, 3: MiddleOuter,
    /// 4: OuterInner, 5: OuterCenter, 6: OuterOuter
    BedButton {
        TopLeft,
        TopRight,
        MiddleInner,
        MiddleCenter,
        MiddleOuter,
        LowerInner,
        LowerCenter,
        LowerOuter,
    }
}

#[derive(Clone, Copy, Debug, defmt::Format, Serialize, Deserialize, MaxSize)]
pub enum LargeBedroom {
    Brightness(f32),
    Temperature(f32),
    Humidity(f32),
    GassResistance(f32), // in Ohm
    Pressure(f32),
    /// parts per million
    Co2(u16),
    /// weight on the left side of the bed
    WeightLeft(u32),
    /// weight on the right side of the bed
    WeightRight(u32),
    DeskButton(DeskButton),
    BedButton(BedButton),
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize)]
pub enum SensorError {
    Sht31(sht31::error::SHTError),
    Bme680(bosch_bme680::BmeError<I2cError>),
    Max44(max44009::Error<I2cError>),
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize)]
pub enum Device {
    Sht31,
    Bme680,
    Max44,
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize)]
pub enum Error {
    Running(SensorError),
    Setup(SensorError),
    SetupTimedOut(Device),
}
