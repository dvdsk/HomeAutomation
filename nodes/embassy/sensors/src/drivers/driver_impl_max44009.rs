use embassy_embedded_hal::shared_bus;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use embassy_stm32::i2c::Master;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use max44009::{Max44009, SlaveAddr};

use crate::errors::SensorError;
use crate::ReInitableDriver;

use super::ConcreteSharedI2c;

impl<'a> ReInitableDriver for Max44009<ConcreteSharedI2c<'a>> {
    type Parts = &'a Mutex<NoopRawMutex, I2c<'static, Async, Master>>;
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
