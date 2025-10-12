use core::time::Duration;

#[cfg(feature = "alloc")]
use crate::affector::{Control, ControlValue};
use crate::button::Press;
use crate::button_enum;
#[cfg(feature = "alloc")]
use crate::reading::tree::{Id, Item, ItemMut, Tree};
#[cfg(feature = "alloc")]
use crate::reading::FloatLabelFormatter;
#[cfg(feature = "alloc")]
use crate::reading::Info;
use crate::shared::impl_is_same_as;
#[cfg(feature = "alloc")]
use crate::{affector, Unit};

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
        Top,
        MiddleInner,
        MiddleCenter,
        MiddleOuter,
        LowerInner,
        LowerCenter,
        LowerOuter,
    }
    crate::Device::LargeBedroom(crate::large_bedroom::Device::Bed(Device::Gpio));
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
#[strum_discriminants(derive(Hash))]
#[repr(u8)]
pub enum Reading {
    Button(Button) = 0,
    Brightness(f32) = 1,
    Temperature(f32) = 2,
    Humidity(f32) = 3,
    GassResistance(f32) = 4, // In Ohm
    Pressure(f32) = 5,
    /// Parts per million
    Co2(u16) = 6,
    /// Weight on the left side of the bed
    WeightLeft(u32) = 7,
    /// Weight on the right side of the bed
    WeightRight(u32) = 8,

    /// Mass Concentration PM1.0 \[μg/m³\]
    MassPm1_0(f32) = 9,
    /// Mass Concentration PM2.5 \[μg/m³\]
    MassPm2_5(f32) = 10,
    /// Mass Concentration PM4.0 \[μg/m³\]
    MassPm4_0(f32) = 11,
    /// Mass Concentration PM10 \[μg/m³\]
    MassPm10(f32) = 12,
    /// Number Concentration PM0.5 \[#/cm³\]
    NumberPm0_5(f32) = 13,
    /// Number Concentration PM1.0 \[#/cm³\]
    NumberPm1_0(f32) = 14,
    /// Number Concentration PM2.5 \[#/cm³\]
    NumberPm2_5(f32) = 15,
    /// Number Concentration PM4.0 \[#/cm³\]
    NumberPm4_0(f32) = 16,
    /// Number Concentration PM10 \[#/cm³\]
    NumberPm10(f32) = 17,
    /// Typical Particle Size8 \[μm\]
    TypicalParticleSize(f32) = 18,
}

#[cfg(feature = "alloc")]
impl Tree for Reading {
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cast_precision_loss)]
    fn inner(&self) -> Item<'_> {
        let (val, device, range, resolution, unit, description) = match self {
            Reading::Brightness(val) => (
                *val,
                Device::Max44.rooted(),
                0.0..=188_000.0,
                0.045,
                Unit::Lux,
                "Brightness",
            ),
            Reading::Temperature(val) => (
                *val,
                Device::Sht31.rooted(),
                -10.0..=45.0,
                0.01,
                Unit::C,
                "Temperature",
            ),
            Reading::Humidity(val) => (
                *val,
                Device::Sht31.rooted(),
                0.0..=100.0,
                0.01,
                Unit::RH,
                "Relative humidity",
            ),
            // air quality: comp_gas = log(R_gas[ohm]) + 0.04 log(Ohm)/%rh * hum[%rh]
            Reading::GassResistance(val) => (
                *val,
                Device::Bme680.rooted(),
                0.0..=1_000_000.0,
                1.0,
                Unit::Ohm,
                "Gass resistance",
            ),
            // record air pressure
            // - 1.081hPa (dead sea 430 below normal sea level)
            // - 870hPa (middle of a tornado)
            Reading::Pressure(val) => (
                *val,
                Device::Bme680.rooted(),
                87_000.0..=108_100.0,
                0.18,
                Unit::Pa,
                "Air pressure",
            ),
            Reading::Co2(val) => (
                f32::from(*val),
                Device::Mhz14.rooted(),
                400.0..=2_000.0,
                1.0,
                Unit::Ppm,
                "Co2 concentration",
            ),
            Reading::WeightLeft(val) => (
                *val as f32,
                Device::Nau7802Left.rooted(),
                0.0..=24.0_f32.exp2(),
                1.0,
                Unit::Ohm,
                "Left weight sensor resistance",
            ),
            Reading::WeightRight(val) => (
                *val as f32,
                Device::Nau7802Right.rooted(),
                0.0..=24.0_f32.exp2(),
                1.0,
                Unit::Ohm,
                "Right weight sensor resistance",
            ),
            Reading::Button(button) => return Item::Node(button),
            Reading::MassPm1_0(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..=1000.0,
                0.1,
                Unit::MicrogramPerM3,
                "Mass concentration particles between 0.3 to 1.0 μm",
            ),
            Reading::MassPm2_5(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..=1000.0,
                0.1,
                Unit::MicrogramPerM3,
                "Mass concentration particles between 0.3 to 2.5 μm",
            ),
            Reading::MassPm4_0(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..=1000.0,
                0.1,
                Unit::MicrogramPerM3,
                "Mass concentration particles between 0.3 to 4.0 μm",
            ),
            Reading::MassPm10(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..=1000.0,
                0.1,
                Unit::MicrogramPerM3,
                "Mass concentration particles between 0.3 to 10.0 μm",
            ),
            Reading::NumberPm0_5(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..=3000.0,
                0.1,
                Unit::NumberPerCm3,
                "Concentration of particles between 0.3 to 0.5 μm",
            ),
            Reading::NumberPm1_0(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..=3000.0,
                0.1,
                Unit::NumberPerCm3,
                "Concentration of particles between 0.3 to 1.0 μm",
            ),
            Reading::NumberPm2_5(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..=3000.0,
                0.1,
                Unit::NumberPerCm3,
                "Concentration of particles between 0.3 to 2.5 μm",
            ),
            Reading::NumberPm4_0(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..=3000.0,
                0.1,
                Unit::NumberPerCm3,
                "Concentration of particles between 0.3 to 4.0 μm",
            ),
            Reading::NumberPm10(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..=3000.0,
                0.1,
                Unit::NumberPerCm3,
                "Concentration of particles between 0.3 to 10.5 μm",
            ),
            Reading::TypicalParticleSize(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..=10_000.0,
                0.5,
                Unit::NanoMeter,
                "Typical particle size",
            ),
        };

        let info = Info {
            val,
            device,
            resolution,
            range,
            unit,
            description,
            branch_id: self.branch_id(),
            label_formatter: Box::new(FloatLabelFormatter),
        };

        Item::Leaf(info)
    }

    fn inner_mut(&mut self) -> ItemMut<'_> {
        use crate::reading::tree::field_as_any;

        let value = field_as_any!(
            self,
            Brightness,
            Temperature,
            Humidity,
            GassResistance,
            Pressure,
            Co2,
            WeightLeft,
            WeightRight,
            MassPm1_0,
            MassPm2_5,
            MassPm4_0,
            MassPm10,
            NumberPm0_5,
            NumberPm1_0,
            NumberPm2_5,
            NumberPm4_0,
            NumberPm10,
            TypicalParticleSize,
            Button
        );
        ItemMut::Leaf(value)
    }

    fn branch_id(&self) -> Id {
        ReadingDiscriminants::from(self) as Id
    }
}

impl_is_same_as! {Reading; Brightness, Temperature, Humidity, GassResistance, Pressure,
    Co2, WeightLeft, WeightRight, MassPm1_0, MassPm2_5, MassPm4_0, MassPm10, NumberPm0_5,
    NumberPm1_0, NumberPm2_5, NumberPm4_0, NumberPm10, TypicalParticleSize;
    (Self::Button(a), Self::Button(b)) => a.is_same_as(b)
}

#[derive(
    Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq,
)]
pub enum Error {
    Running(SensorError),
    Setup(SensorError),
    SetupTimedOut(Device),
    Timeout(Device),
}

impl Error {
    #[must_use]
    pub(crate) fn device(&self) -> Device {
        match self {
            Self::Running(sensor_err) | Self::Setup(sensor_err) => {
                sensor_err.device()
            }
            Self::SetupTimedOut(device) | Self::Timeout(device) => {
                device.clone()
            }
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Running(e) => write!(f, "{e}"),
            Error::Setup(e) => write!(f, "setup: {e}"),
            Error::SetupTimedOut(_) => write!(f, "setup timed out"),
            Error::Timeout(_) => write!(f, "timed out running"),
        }
    }
}

#[derive(
    Clone, Debug, defmt::Format, Serialize, Deserialize, Eq, PartialEq,
)]
pub enum SensorError {
    Sht31(heapless::String<200>),
    Bme680(heapless::String<200>),
    Max44(heapless::String<200>),
    Mhz14(heapless::String<200>),
    Sps30(heapless::String<200>),
    Button(heapless::String<200>),
    Nau7802Left(heapless::String<200>),
    Nau7802Right(heapless::String<200>),
}

impl core::fmt::Display for SensorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SensorError::Sht31(e)
            | SensorError::Bme680(e)
            | SensorError::Max44(e)
            | SensorError::Mhz14(e)
            | SensorError::Sps30(e)
            | SensorError::Button(e)
            | SensorError::Nau7802Left(e)
            | SensorError::Nau7802Right(e) => write!(f, "{e}"),
        }
    }
}

impl MaxSize for SensorError {
    const POSTCARD_MAX_SIZE: usize = 201;
}

impl SensorError {
    #[must_use]
    pub fn device(&self) -> Device {
        match self {
            SensorError::Sht31(_) => Device::Sht31,
            SensorError::Bme680(_) => Device::Bme680,
            SensorError::Max44(_) => Device::Max44,
            SensorError::Mhz14(_) => Device::Mhz14,
            SensorError::Sps30(_) => Device::Sps30,
            SensorError::Button(_) => Device::Gpio,
            SensorError::Nau7802Left(_) => Device::Nau7802Right,
            SensorError::Nau7802Right(_) => Device::Nau7802Right,
        }
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
pub enum Device {
    Sht31,
    Bme680,
    Max44,
    Mhz14,
    Sps30,
    Nau7802Right,
    Nau7802Left,
    Gpio,
}

impl core::fmt::Display for Device {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.info().name)
    }
}

macro_rules! tree {
    [$thing:ident; $($item:expr),+] => {
        [$(crate::$thing::LargeBedroom(
            crate::large_bedroom::$thing::Bed($item),
        )),+]
    };
}

impl Device {
    /// Note the order in which the `affects_readings` occur is the order in which
    /// they will be stored, do not change it!
    #[must_use]
    pub const fn info(&self) -> crate::DeviceInfo {
        let min_sample_interval = Duration::from_secs(5);
        let max_sample_interval = Duration::from_secs(5);
        let temporal_resolution = Duration::from_secs(1);
        match self {
            Device::Sht31 => crate::DeviceInfo {
                name: "Sht31",
                affects_readings: &tree![Reading; Reading::Temperature(0.0), Reading::Humidity(0.0)],
                temporal_resolution,
                min_sample_interval,
                max_sample_interval,
                affectors: &[],
            },
            Device::Bme680 => crate::DeviceInfo {
                name: "Bme680",
                affects_readings: &tree![Reading;
                    Reading::GassResistance(0.0),
                    Reading::Pressure(0.0)
                ],
                temporal_resolution,
                min_sample_interval,
                max_sample_interval,
                affectors: &[],
            },
            Device::Max44 => crate::DeviceInfo {
                name: "Max44",
                affects_readings: &tree![Reading; Reading::Brightness(0.0)],
                temporal_resolution: Duration::from_millis(50),
                min_sample_interval: Duration::from_millis(50),
                max_sample_interval,
                affectors: &[],
            },
            Device::Mhz14 => crate::DeviceInfo {
                name: "Mhz14",
                affects_readings: &tree![Reading; Reading::Co2(0)],
                temporal_resolution,
                min_sample_interval,
                max_sample_interval,
                affectors: &tree!(Affector; Affector::MhzZeroPointCalib),
            },
            Device::Sps30 => crate::DeviceInfo {
                name: "Sps30",
                affects_readings: &tree![Reading;
                    Reading::MassPm1_0(0.0),
                    Reading::MassPm2_5(0.0),
                    Reading::MassPm4_0(0.0),
                    Reading::MassPm10(0.0),
                    Reading::NumberPm0_5(0.0),
                    Reading::NumberPm1_0(0.0),
                    Reading::NumberPm2_5(0.0),
                    Reading::NumberPm4_0(0.0),
                    Reading::NumberPm10(0.0),
                    Reading::TypicalParticleSize(0.0)
                ],
                temporal_resolution,
                min_sample_interval,
                max_sample_interval,
                affectors: &tree!(Affector; Affector::Sps30FanClean),
            },
            Device::Nau7802Left => crate::DeviceInfo {
                name: "Nau7802Left",
                affects_readings: &tree![Reading; Reading::WeightLeft(0)],
                temporal_resolution: Duration::from_millis(100),
                min_sample_interval: Duration::from_millis(100),
                max_sample_interval,
                affectors: &tree!(Affector; Affector::Nau7802LeftCalib),
            },
            Device::Nau7802Right => crate::DeviceInfo {
                name: "Nau7802Right",
                affects_readings: &tree![Reading; Reading::WeightRight(0)],
                temporal_resolution: Duration::from_millis(100),
                min_sample_interval: Duration::from_millis(100),
                max_sample_interval,
                affectors: &tree!(Affector; Affector::Nau7802RightCalib),
            },
            Device::Gpio => crate::DeviceInfo {
                affects_readings: &tree![Reading;
                    Reading::Button(Button::Top(Press(0))),
                    Reading::Button(Button::MiddleInner(Press(0))),
                    Reading::Button(Button::MiddleCenter(Press(0))),
                    Reading::Button(Button::MiddleOuter(Press(0))),
                    Reading::Button(Button::LowerInner(Press(0))),
                    Reading::Button(Button::LowerCenter(Press(0))),
                    Reading::Button(Button::LowerOuter(Press(0)))
                ],
                ..crate::DeviceInfo::button_defaults()
            },
        }
    }

    #[cfg(feature = "alloc")]
    #[must_use]
    const fn rooted(self) -> crate::Device {
        crate::Device::LargeBedroom(crate::large_bedroom::Device::Bed(self))
    }
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
    Eq,
    Hash,
)]
#[strum_discriminants(derive(Hash))]
pub enum Affector {
    Sps30FanClean,
    MhzZeroPointCalib,
    Nau7802LeftCalib,
    Nau7802RightCalib,
    RgbLed { red: u8, green: u8, blue: u8 },
    ResetNode,
}

impl crate::IsSameAs for Affector {
    fn is_same_as(&self, other: &Self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match (self, other) {
            (Affector::Sps30FanClean, Affector::Sps30FanClean) => true,
            (Affector::MhzZeroPointCalib, Affector::MhzZeroPointCalib) => true,
            (Affector::Nau7802LeftCalib, Affector::Nau7802LeftCalib) => true,
            (Affector::Nau7802RightCalib, Affector::Nau7802RightCalib) => true,
            (Affector::RgbLed { .. }, Affector::RgbLed { .. }) => true,
            (Affector::ResetNode, Affector::ResetNode) => true,
            _ => false,
        }
    }
}

impl Affector {
    #[cfg(feature = "alloc")]
    pub fn controls(&mut self) -> Vec<Control<'_>> {
        match self {
            Affector::Sps30FanClean => vec![Control {
                name: "start fan cleaning",
                value: ControlValue::Trigger,
            }],
            Affector::MhzZeroPointCalib => vec![Control {
                name: "calibrate current reading as 400 ppm",
                value: ControlValue::Trigger,
            }],
            Affector::Nau7802LeftCalib => vec![Control {
                name: "things with ac power noise or something",
                value: ControlValue::Trigger,
            }],
            Affector::Nau7802RightCalib => vec![Control {
                name: "things with ac power noise or something",
                value: ControlValue::Trigger,
            }],
            Affector::RgbLed { red, green, blue } => vec![
                Control {
                    name: "red",
                    value: ControlValue::SetNum {
                        valid_range: 0..255,
                        value: *red as usize,
                        setter: Some(Box::new(|input: usize| {
                            *red = input as u8
                        })),
                    },
                },
                Control {
                    name: "green",
                    value: ControlValue::SetNum {
                        valid_range: 0..255,
                        value: *green as usize,
                        setter: Some(Box::new(|input: usize| {
                            *green = input as u8
                        })),
                    },
                },
                Control {
                    name: "blue",
                    value: ControlValue::SetNum {
                        valid_range: 0..255,
                        value: *blue as usize,
                        setter: Some(Box::new(|input: usize| {
                            *blue = input as u8
                        })),
                    },
                },
            ],
            Affector::ResetNode => vec![Control {
                name: "reset the node",
                value: ControlValue::Trigger,
            }],
        }
    }
}

#[cfg(feature = "alloc")]
impl affector::tree::Tree for Affector {
    fn inner(&self) -> affector::tree::Item<'_> {
        let description = match self {
            Affector::Sps30FanClean => "Accelerate the fan to maximum speed for 10 seconds in order to blow out the dust accumulated in the fan",
            Affector::MhzZeroPointCalib => "Set the current co2 value as 400ppm",
            Affector::Nau7802LeftCalib |
            Affector::Nau7802RightCalib =>  "Detect and correct power supply and temperature variations to ADC",
            Affector::RgbLed {..} => "Set color & power of the RGB led at the top of the bed post",
            Affector::ResetNode => "Reset the node, this might fix errors such as I2c getting stuck after Arbitration Error",
        };

        affector::tree::Item::Leaf(affector::Info { description })
    }

    fn branch_id(&self) -> affector::tree::Id {
        AffectorDiscriminants::from(self) as Id
    }
}
