use bosch_bme680::Bme680;
use embassy_embedded_hal::shared_bus;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Timer};
use max44009::{Max44009, SlaveAddr};
use sht31::mode::SingleShot;
use sht31::SHT31;

use super::concrete_types::ConcreteSharedI2c;
use crate::error_cache::{Error, SensorError};

impl<'a> Driver for Max44009<ConcreteSharedI2c<'a>> {
    type Parts = &'a Mutex<NoopRawMutex, I2c<'static, Async>>;
    type Measurement = f32;

    #[inline(always)]
    async fn init(parts: Self::Parts) -> Result<Self, SensorError> {
        let shared_i2c = shared_bus::asynch::i2c::I2cDevice::new(parts);
        // wrap these two into something returning Ok or Err
        // then make it a generic arg, maybe Driverfactory::init
        let mut driver = Max44009::new(shared_i2c, SlaveAddr::default());
        driver
            .set_measurement_mode(max44009::MeasurementMode::Continuous)
            .await
            .map_err(SensorError::Max44)?;
        Ok(driver)
    }

    #[inline(always)]
    async fn measure(&mut self) -> Result<Self::Measurement, SensorError> {
        self.read_lux().await.map_err(SensorError::Max44)
    }
}

pub type Max44Driver<'a> = ReInitOnErrorDriver<Max44009<ConcreteSharedI2c<'a>>>;

impl<'a> Driver for Bme680<ConcreteSharedI2c<'a>, Delay> {
    type Parts = &'a Mutex<NoopRawMutex, I2c<'static, Async>>;
    type Measurement = bosch_bme680::MeasurementData;

    #[inline(always)]
    async fn init(parts: Self::Parts) -> Result<Self, SensorError> {
        let shared_i2c = shared_bus::asynch::i2c::I2cDevice::new(parts);
        Bme680::new(
            shared_i2c,
            bosch_bme680::DeviceAddress::Secondary,
            Delay,
            &bosch_bme680::Configuration::default(),
            21,
        )
        .await
        .map_err(SensorError::Bme680)
    }

    #[inline(always)]
    async fn measure(&mut self) -> Result<Self::Measurement, SensorError> {
        self.measure().await.map_err(SensorError::Bme680)
    }
}

pub type Bme680Driver<'a> = ReInitOnErrorDriver<Bme680<ConcreteSharedI2c<'a>, Delay>>;

impl<'a> Driver for SHT31<SingleShot, ConcreteSharedI2c<'a>> {
    type Parts = &'a Mutex<NoopRawMutex, I2c<'static, Async>>;
    type Measurement = sht31::Reading;

    #[inline(always)]
    async fn init(parts: Self::Parts) -> Result<Self, SensorError> {
        let shared_i2c = shared_bus::asynch::i2c::I2cDevice::new(parts);
        Ok(SHT31::new(shared_i2c, Delay)
            .with_mode(SingleShot)
            .with_unit(sht31::TemperatureUnit::Celsius)
            .with_accuracy(sht31::Accuracy::High))
    }

    #[inline(always)]
    async fn measure(&mut self) -> Result<Self::Measurement, SensorError> {
        use sht31::mode::Sht31Measure;
        use sht31::mode::Sht31Reader;

        Sht31Measure::measure(self)
            .await
            .map_err(SensorError::Sht31)?;
        Timer::after_secs(1).await;
        self.read().await.map_err(SensorError::Sht31)
    }
}

pub type Sht31Driver<'a> = ReInitOnErrorDriver<SHT31<SingleShot, ConcreteSharedI2c<'a>>>;

pub trait Driver: Sized {
    type Parts: Clone;
    type Measurement;

    async fn init(parts: Self::Parts) -> Result<Self, SensorError>;
    async fn measure(&mut self) -> Result<Self::Measurement, SensorError>;
}

pub enum ReInitOnErrorDriver<D>
where
    D: Driver,
{
    Ready { driver: D, parts: D::Parts },
    Uninit { parts: D::Parts },
}

impl<D: Driver> ReInitOnErrorDriver<D> {
    pub fn new(parts: D::Parts) -> Self {
        Self::Uninit { parts }
    }

    pub async fn try_measure(&mut self) -> Result<D::Measurement, Error> {
        let parts = match self {
            Self::Ready { parts, .. } => parts,
            Self::Uninit { parts } => parts,
        }
        .clone();

        let mut owned_self = Self::Uninit { parts };
        core::mem::swap(&mut owned_self, self);
        let (mut new_self, res) = owned_self.advance_state().await;
        core::mem::swap(&mut new_self, self);
        res
    }

    #[inline(always)]
    async fn advance_state(self) -> (Self, Result<D::Measurement, Error>) {
        match self {
            Self::Ready {
                mut driver,
                parts,
                // make a trait over driver that has a measure method
            } => match driver.measure().await {
                Ok(val) => {
                    let new_self = Self::Ready { driver, parts };
                    (new_self, Ok(val))
                }
                Err(err) => {
                    let new_self = Self::Uninit { parts };
                    (new_self, Err(Error::Running(err)))
                }
            },
            Self::Uninit { parts } => {
                match D::init(parts.clone()).await {
                    Ok(mut driver) => {
                        // uses driver::measure again, make that return a SensorError
                        match driver.measure().await {
                            Ok(val) => {
                                let new_self = Self::Ready { driver, parts };
                                (new_self, Ok(val))
                            }
                            Err(err) => {
                                let new_self = Self::Uninit { parts };
                                let err = Error::Running(err);
                                (new_self, Err(err))
                            }
                        }
                    }
                    Err(err) => {
                        let new_self = Self::Uninit { parts };
                        let err = Error::Setup(err);
                        (new_self, Err(err))
                    }
                }
            }
        }
    }
}
