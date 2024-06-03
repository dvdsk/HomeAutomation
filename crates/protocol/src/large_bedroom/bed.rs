use crate::downcast_err::{I2cError, UartError};
#[cfg(feature = "alloc")]
use crate::{Tomato, TomatoItem};
use crate::button_enum;

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

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
    Button {
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

#[derive(
    strum::EnumDiscriminants, Clone, Copy, Debug, defmt::Format, Serialize, Deserialize, MaxSize,
)]
#[strum_discriminants(derive(Hash))]
pub enum Reading {
    Button(Button),
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

    /// Mass Concentration PM1.0 \[μg/m³\]
    MassPm1_0(f32),
    /// Mass Concentration PM2.5 \[μg/m³\]
    MassPm2_5(f32),
    /// Mass Concentration PM4.0 \[μg/m³\]
    MassPm4_0(f32),
    /// Mass Concentration PM10 \[μg/m³\]
    MassPm10(f32),
    /// Number Concentration PM0.5 \[#/cm³\]
    MassPm0_5(f32),
    /// Number Concentration PM1.0 \[#/cm³\]
    NumberPm1_0(f32),
    /// Number Concentration PM2.5 \[#/cm³\]
    NumberPm2_5(f32),
    /// Number Concentration PM4.0 \[#/cm³\]
    NumberPm4_0(f32),
    /// Number Concentration PM10 \[#/cm³\]
    NumberPm10(f32),
    /// Typical Particle Size8 \[μm\]
    TypicalParticleSize(f32),
}

#[cfg(feature = "alloc")]
impl Tomato for Reading {
    fn inner<'a>(&'a self) -> TomatoItem<'a> {
        let val = match self {
            Reading::Brightness(val) => *val,
            Reading::Temperature(val) => *val,
            Reading::Humidity(val) => *val,
            Reading::GassResistance(val) => *val,
            Reading::Pressure(val) => *val,
            Reading::Co2(val) => *val as f32,
            Reading::WeightLeft(val) => *val as f32,
            Reading::WeightRight(val) => *val as f32,
            Reading::Button(val) => (*val).into(),
            Reading::MassPm1_0(val) => *val,
            Reading::MassPm2_5(val) => *val,
            Reading::MassPm4_0(val) => *val,
            Reading::MassPm10(val) => *val,
            Reading::MassPm0_5(val) => *val,
            Reading::NumberPm1_0(val) => *val,
            Reading::NumberPm2_5(val) => *val,
            Reading::NumberPm4_0(val) => *val,
            Reading::NumberPm10(val) => *val,
            Reading::TypicalParticleSize(val) => *val,
        };
        TomatoItem::Leaf(val)
    }

    fn id(&self) -> crate::TomatoId {
        ReadingDiscriminants::from(self) as crate::TomatoId
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
pub enum Error {
    Running(SensorError),
    Setup(SensorError),
    SetupTimedOut(Device),
    Timeout(Device),
}

impl Error {
    pub fn broken_readings(&self) -> &[ReadingDiscriminants] {
        match self {
            Self::Running(sensor_err) | Self::Setup(sensor_err) => sensor_err.broken_readings(),
            Self::SetupTimedOut(device) | Self::Timeout(device) => device.broken_readings(),
        }
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
pub enum SensorError {
    Sht31(sht31::error::SHTError),
    Bme680(bosch_bme680::BmeError<I2cError>),
    Max44(max44009::Error<I2cError>),
    Mhz14(mhzx::Error<UartError, UartError>),
    Sps30(sps30_async::Error<UartError, UartError>),
}

impl SensorError {
    pub fn broken_readings(&self) -> &[ReadingDiscriminants] {
        let device = match self {
            SensorError::Sht31(_) => Device::Sht31,
            SensorError::Bme680(_) => Device::Bme680,
            SensorError::Max44(_) => Device::Max44,
            SensorError::Mhz14(_) => Device::Mhz14,
            SensorError::Sps30(_) => Device::Sps30,
        };
        device.broken_readings()
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
pub enum Device {
    Sht31,
    Bme680,
    Max44,
    Mhz14,
    Sps30,
}

impl Device {
    pub fn broken_readings(&self) -> &'static [ReadingDiscriminants] {
        match self {
            Device::Sht31 => &[
                ReadingDiscriminants::Temperature,
                ReadingDiscriminants::Humidity,
            ],
            Device::Bme680 => &[
                ReadingDiscriminants::GassResistance,
                ReadingDiscriminants::Pressure,
            ],
            Device::Max44 => &[ReadingDiscriminants::Brightness],
            Device::Mhz14 => &[ReadingDiscriminants::Co2],
            Device::Sps30 => &[
                ReadingDiscriminants::MassPm1_0,
                ReadingDiscriminants::MassPm2_5,
                ReadingDiscriminants::MassPm4_0,
                ReadingDiscriminants::MassPm10,
                ReadingDiscriminants::MassPm0_5,
                ReadingDiscriminants::NumberPm1_0,
                ReadingDiscriminants::NumberPm2_5,
                ReadingDiscriminants::NumberPm4_0,
                ReadingDiscriminants::NumberPm10,
                ReadingDiscriminants::TypicalParticleSize,
            ],
        }
    }
}
