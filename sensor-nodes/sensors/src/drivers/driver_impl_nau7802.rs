use embassy_embedded_hal::shared_bus;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::{Async, Blocking};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Delay;
use nau7802_async::Nau7802;

use crate::errors::SensorError;
use crate::ReInitableDriver;

use super::{ConcreteBlockingI2c, ConcreteSharedI2c, I2cWrapper};

impl<'a> ReInitableDriver for Nau7802<ConcreteSharedI2c<'a>, Delay> {
    type Parts = &'a Mutex<NoopRawMutex, I2c<'static, Async>>;
    type Measurement = u32;

    async fn init(parts: Self::Parts) -> Result<Self, SensorError> {
        let shared_i2c = shared_bus::asynch::i2c::I2cDevice::new(parts);
        // Wrap these two into something returning Ok or Err
        // then make it a generic arg, maybe Driverfactory::init
        let driver = Nau7802::new(shared_i2c, Delay)
            .await
            .map_err(SensorError::Nau7802Left)?;
        Ok(driver)
    }

    async fn measure(&mut self) -> Result<Self::Measurement, SensorError> {
        self.read().await.map_err(SensorError::Nau7802Left)
    }
}

impl<'a> ReInitableDriver for Nau7802<ConcreteBlockingI2c<'a>, Delay> {
    type Parts = &'a I2cWrapper<embassy_stm32::i2c::I2c<'static, Blocking>>;
    type Measurement = u32;

    async fn init(parts: Self::Parts) -> Result<Self, SensorError> {
        let shared_i2c = parts;
        // Wrap these two into something returning Ok or Err
        // then make it a generic arg, maybe Driverfactory::init
        let driver = Nau7802::new(shared_i2c, Delay)
            .await
            .map_err(SensorError::Nau7802Left)?;
        Ok(driver)
    }

    async fn measure(&mut self) -> Result<Self::Measurement, SensorError> {
        self.read().await.map_err(SensorError::Nau7802Left)
    }
}
