use embassy_embedded_hal::shared_bus;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Timer};
use sht31::mode::SingleShot;
use sht31::SHT31;

use crate::errors::SensorError;
use crate::ReInitableDriver;

use super::ConcreteSharedI2c;

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
