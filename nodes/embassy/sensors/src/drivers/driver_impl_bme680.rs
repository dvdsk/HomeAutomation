use bosch_bme680::Bme680;
use embassy_embedded_hal::shared_bus;
use embassy_stm32::i2c::I2c;
use embassy_stm32::i2c::Master;
use embassy_stm32::mode::Async;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Delay;

use crate::errors::SensorError;
use crate::ReInitableDriver;

use super::ConcreteSharedI2c;

impl<'a> ReInitableDriver for Bme680<ConcreteSharedI2c<'a>, Delay> {
    type Parts = &'a Mutex<NoopRawMutex, I2c<'static, Async, Master>>;
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
