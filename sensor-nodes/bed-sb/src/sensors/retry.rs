use bosch_bme680::Bme680;
use embassy_embedded_hal::shared_bus;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Timer};
use max44009::{Max44009, SlaveAddr};
use nau7802_async::Nau7802;
use sht31::mode::SingleShot;
use sht31::SHT31;

use super::concrete_types::ConcreteSharedI2c;
use crate::error_cache::SensorError;

mod reinit_on_error;
pub use reinit_on_error::{
    Bme680Driver, Max44Driver, Nau7802Driver, Sht31Driver,
};

mod retry_init;
pub use retry_init::Sps30Driver;

impl<'a> ReInitableDriver for SHT31<SingleShot, ConcreteSharedI2c<'a>> {
    type Parts = &'a Mutex<NoopRawMutex, I2c<'static, Async>>;
    type Measurement = sht31::Reading;

    async fn init(parts: Self::Parts) -> Result<Self, SensorError> {
        let shared_i2c = shared_bus::asynch::i2c::I2cDevice::new(parts);
        Ok(SHT31::new(shared_i2c, Delay)
            .with_mode(SingleShot)
            .with_unit(sht31::TemperatureUnit::Celsius)
            .with_accuracy(sht31::Accuracy::High))
    }

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

impl<'a> ReInitableDriver for Bme680<ConcreteSharedI2c<'a>, Delay> {
    type Parts = &'a Mutex<NoopRawMutex, I2c<'static, Async>>;
    type Measurement = bosch_bme680::MeasurementData;

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

    async fn measure(&mut self) -> Result<Self::Measurement, SensorError> {
        self.measure().await.map_err(SensorError::Bme680)
    }
}

impl<'a> ReInitableDriver for Max44009<ConcreteSharedI2c<'a>> {
    type Parts = &'a Mutex<NoopRawMutex, I2c<'static, Async>>;
    type Measurement = f32;

    async fn init(parts: Self::Parts) -> Result<Self, SensorError> {
        let shared_i2c = shared_bus::asynch::i2c::I2cDevice::new(parts);
        // Wrap these two into something returning Ok or Err
        // then make it a generic arg, maybe Driverfactory::init
        let mut driver = Max44009::new(shared_i2c, SlaveAddr::default());
        driver
            .set_measurement_mode(max44009::MeasurementMode::Continuous)
            .await
            .map_err(SensorError::Max44)?;
        Ok(driver)
    }
    async fn measure(&mut self) -> Result<Self::Measurement, SensorError> {
        self.read_lux().await.map_err(SensorError::Max44)
    }
}

impl<'a> ReInitableDriver for Nau7802<ConcreteSharedI2c<'a>, Delay> {
    type Parts = &'a Mutex<NoopRawMutex, I2c<'static, Async>>;
    type Measurement = u32;

    async fn init(parts: Self::Parts) -> Result<Self, SensorError> {
        let shared_i2c = shared_bus::asynch::i2c::I2cDevice::new(parts);
        // Wrap these two into something returning Ok or Err
        // then make it a generic arg, maybe Driverfactory::init
        let driver = Nau7802::new(shared_i2c, Delay)
            .await
            .map_err(SensorError::Nau7802)?;
        Ok(driver)
    }

    async fn measure(&mut self) -> Result<Self::Measurement, SensorError> {
        self.read().await.map_err(SensorError::Nau7802)
    }
}

/// A driver that can be re-initialized from copy-able parts
pub trait ReInitableDriver: Sized {
    type Parts: Clone;
    type Measurement;

    async fn init(parts: Self::Parts) -> Result<Self, SensorError>;
    async fn measure(&mut self) -> Result<Self::Measurement, SensorError>;
}

pub trait Driver {
    type Measurement;
    type Affector;
    async fn try_measure(&mut self) -> Result<Self::Measurement, crate::error_cache::Error>;
    /// For example calibrate, run sensor cleaning etc
    async fn affect(&mut self, _: Self::Affector) -> Result<(), crate::error_cache::Error> {
        Ok(())
    }
    fn device(&self) -> protocol::small_bedroom::bed::Device;
}
