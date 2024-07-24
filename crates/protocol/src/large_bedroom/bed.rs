use core::time::Duration;

use crate::button::Press;
use crate::button_enum;
#[cfg(feature = "alloc")]
use crate::reading_tree::{Id, Item, ReadingInfo, Tree};
#[cfg(feature = "alloc")]
use crate::Unit;

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
    NumberPm0_5(f32),
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
impl Tree for Reading {
    #[must_use]
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::cast_precision_loss)]
    fn inner(&self) -> Item<'_> {
        let (val, device, range, resolution, unit, description) = match self {
            Reading::Brightness(val) => (
                *val,
                Device::Max44.rooted(),
                0.045..188.000,
                0.045,
                Unit::Lux,
                "Brightness",
            ),
            Reading::Temperature(val) => (
                *val,
                Device::Sht31.rooted(),
                -10.0..45.0,
                0.01,
                Unit::C,
                "Temperature",
            ),
            Reading::Humidity(val) => (
                *val,
                Device::Sht31.rooted(),
                0.0..100.0,
                0.01,
                Unit::RH,
                "Relative humidity",
            ),
            // air quality: comp_gas = log(R_gas[ohm]) + 0.04 log(Ohm)/%rh * hum[%rh]
            Reading::GassResistance(val) => (
                *val,
                Device::Bme680.rooted(),
                0.0..1_000_000.0,
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
                87_000.0..108_100.0,
                0.18,
                Unit::Pa,
                "Air pressure",
            ),
            Reading::Co2(val) => (
                f32::from(*val),
                Device::Mhz14.rooted(),
                400.0..2_000.0,
                1.0,
                Unit::Ppm,
                "Co2 concentration",
            ),
            Reading::WeightLeft(val) => (
                *val as f32,
                Device::Nau7802Left.rooted(),
                0.0..2.0_f32.exp2(),
                1.0,
                Unit::Ohm,
                "Left weight sensor resistance",
            ),
            Reading::WeightRight(val) => (
                *val as f32,
                Device::Nau7802Right.rooted(),
                0.0..2.0_f32.exp2(),
                1.0,
                Unit::Ohm,
                "Right weight sensor resistance",
            ),
            Reading::Button(button) => return button.inner(),
            Reading::MassPm1_0(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..1000.0,
                0.1,
                Unit::MicrogramPerM3,
                "Mass concentration particles between 0.3 to 1.0 μm",
            ),
            Reading::MassPm2_5(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..1000.0,
                0.1,
                Unit::MicrogramPerM3,
                "Mass concentration particles between 0.3 to 2.5 μm",
            ),
            Reading::MassPm4_0(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..1000.0,
                0.1,
                Unit::MicrogramPerM3,
                "Mass concentration particles between 0.3 to 4.0 μm",
            ),
            Reading::MassPm10(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..1000.0,
                0.1,
                Unit::MicrogramPerM3,
                "Mass concentration particles between 0.3 to 10.0 μm",
            ),
            Reading::NumberPm0_5(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..3000.0,
                0.1,
                Unit::NumberPerCm3,
                "Concentration of particles between 0.3 to 0.5 μm",
            ),
            Reading::NumberPm1_0(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..3000.0,
                0.1,
                Unit::NumberPerCm3,
                "Concentration of particles between 0.3 to 1.0 μm",
            ),
            Reading::NumberPm2_5(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..3000.0,
                0.1,
                Unit::NumberPerCm3,
                "Concentration of particles between 0.3 to 2.5 μm",
            ),
            Reading::NumberPm4_0(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..3000.0,
                0.1,
                Unit::NumberPerCm3,
                "Concentration of particles between 0.3 to 4.0 μm",
            ),
            Reading::NumberPm10(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..3000.0,
                0.1,
                Unit::NumberPerCm3,
                "Concentration of particles between 0.3 to 10.5 μm",
            ),
            Reading::TypicalParticleSize(val) => (
                *val,
                Device::Sps30.rooted(),
                0.0..10_000.0,
                0.5,
                Unit::NanoMeter,
                "Typical particle size",
            ),
        };

        Item::Leaf(ReadingInfo {
            val,
            device,
            resolution,
            range,
            unit,
            description,
            branch_id: self.branch_id(),
        })
    }

    #[must_use]
    fn branch_id(&self) -> Id {
        ReadingDiscriminants::from(self) as Id
    }
}

impl Reading {
    #[must_use]
    pub fn is_same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Brightness(_), Self::Brightness(_))
            | (Self::Temperature(_), Self::Temperature(_))
            | (Self::Humidity(_), Self::Humidity(_))
            | (Self::GassResistance(_), Self::GassResistance(_))
            | (Self::Pressure(_), Self::Pressure(_))
            | (Self::Co2(_), Self::Co2(_))
            | (Self::WeightLeft(_), Self::WeightLeft(0))
            | (Self::WeightRight(_), Self::WeightRight(0))
            | (Self::MassPm1_0(_), Self::MassPm1_0(_))
            | (Self::MassPm2_5(_), Self::MassPm2_5(_))
            | (Self::MassPm4_0(_), Self::MassPm4_0(_))
            | (Self::MassPm10(_), Self::MassPm10(_))
            | (Self::NumberPm0_5(_), Self::NumberPm0_5(_))
            | (Self::NumberPm1_0(_), Self::NumberPm1_0(_))
            | (Self::NumberPm2_5(_), Self::NumberPm2_5(_))
            | (Self::NumberPm4_0(_), Self::NumberPm4_0(_))
            | (Self::NumberPm10(_), Self::NumberPm10(_))
            | (Self::TypicalParticleSize(_), Self::TypicalParticleSize(_)) => true,
            (Self::Button(a), Self::Button(b)) => a.is_same_as(b),
            _ => false,
        }
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
    #[must_use]
    pub(crate) fn device(&self) -> Device {
        match self {
            Self::Running(sensor_err) | Self::Setup(sensor_err) => sensor_err.device(),
            Self::SetupTimedOut(device) | Self::Timeout(device) => device.clone(),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Running(e) => write!(f, "{} ran into error: {e}", e.device()),
            Error::Setup(e) => write!(f, "{} errored during setup: {e}", e.device()),
            Error::SetupTimedOut(d) => write!(f, "{d} timed out during setup"),
            Error::Timeout(d) => write!(f, "{d} timed out running"),
        }
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, Eq, PartialEq)]
pub enum SensorError {
    Sht31(heapless::String<200>),
    Bme680(heapless::String<200>),
    Max44(heapless::String<200>),
    Mhz14(heapless::String<200>),
    Sps30(heapless::String<200>),
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
            SensorError::Nau7802Left(_) => Device::Nau7802Left,
            SensorError::Nau7802Right(_) => Device::Nau7802Right,
        }
    }
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq, Hash)]
pub enum Device {
    Sht31,
    Bme680,
    Max44,
    Mhz14,
    Sps30,
    Nau7802Left,
    Nau7802Right,
    Gpio,
}

impl core::fmt::Display for Device {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.info().name)
    }
}

macro_rules! tree {
    [$($reading:expr),+] => {
        [$(crate::Reading::LargeBedroom(
            crate::large_bedroom::Reading::Bed($reading),
        )),+]
    };
}

impl Device {
    /// Note the order in which the affects_readings occur is the order in which
    /// they will be stored, do not change it!
    #[must_use]
    pub const fn info(&self) -> crate::DeviceInfo {
        let min_sample_interval = Duration::from_secs(5);
        let temporal_resolution = Duration::from_secs(1);
        match self {
            Device::Sht31 => crate::DeviceInfo {
                name: "Sht31",
                affects_readings: &tree![Reading::Temperature(0.0), Reading::Humidity(0.0)],
                temporal_resolution,
                min_sample_interval,
            },
            Device::Bme680 => crate::DeviceInfo {
                name: "Bme680",
                affects_readings: &tree![Reading::GassResistance(0.0), Reading::Pressure(0.0)],
                temporal_resolution,
                min_sample_interval,
            },
            Device::Max44 => crate::DeviceInfo {
                name: "Max44",
                affects_readings: &tree![Reading::Brightness(0.0)],
                temporal_resolution: Duration::from_millis(50),
                min_sample_interval: Duration::from_millis(50),
            },
            Device::Mhz14 => crate::DeviceInfo {
                name: "Mhz14",
                affects_readings: &tree![Reading::Co2(0)],
                temporal_resolution,
                min_sample_interval,
            },
            Device::Sps30 => crate::DeviceInfo {
                name: "Sps30",
                affects_readings: &tree![
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
            },
            Device::Nau7802Left => crate::DeviceInfo {
                name: "Nau7802Left",
                affects_readings: &tree![Reading::WeightLeft(0)],
                temporal_resolution: Duration::from_millis(100),
                min_sample_interval: Duration::from_millis(100),
            },
            Device::Nau7802Right => crate::DeviceInfo {
                name: "Nau7802Right",
                affects_readings: &tree![Reading::WeightRight(0)],
                temporal_resolution,
                min_sample_interval,
            },
            Device::Gpio => crate::DeviceInfo {
                name: "Gpio",
                affects_readings: &tree![
                    Reading::Button(Button::TopLeft(Press(0))),
                    Reading::Button(Button::TopRight(Press(0))),
                    Reading::Button(Button::MiddleInner(Press(0))),
                    Reading::Button(Button::MiddleCenter(Press(0))),
                    Reading::Button(Button::MiddleOuter(Press(0))),
                    Reading::Button(Button::LowerInner(Press(0))),
                    Reading::Button(Button::LowerCenter(Press(0))),
                    Reading::Button(Button::LowerOuter(Press(0)))
                ],
                temporal_resolution: Duration::from_millis(1),
                min_sample_interval: Duration::from_millis(2),
            },
        }
    }

    #[cfg(feature = "alloc")]
    #[must_use]
    fn rooted(self) -> crate::Device {
        crate::Device::LargeBedroom(crate::large_bedroom::Device::Bed(self))
    }
}
