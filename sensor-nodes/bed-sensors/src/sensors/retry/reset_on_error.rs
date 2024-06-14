// use bosch_bme680::Bme680;
// use embassy_embedded_hal::shared_bus;
// use embassy_stm32::i2c::I2c;
// use embassy_stm32::mode::Async;
// use embassy_sync::blocking_mutex::raw::NoopRawMutex;
// use embassy_sync::mutex::Mutex;
// use embassy_time::{Delay, Timer};
// use max44009::{Max44009, SlaveAddr};
// use sht31::mode::SingleShot;
// use sht31::SHT31;
//
use super::super::concrete_types::{ConcreteRx, ConcreteTx};

use crate::error_cache::{Error, SensorError};

pub trait ResettableDriver: Sized {
    type Measurement;

    async fn reset(&mut self) -> Result<Self, SensorError>;
    async fn measure(&mut self) -> Result<Self::Measurement, SensorError>;
}

pub struct ResetOnErrorDriver<D>
where
    D: ResettableDriver,
{
    driver: D,
}

impl<D: ResettableDriver> ResetOnErrorDriver<D> {
    pub fn new(driver: D) -> Self {
        Self { driver }
    }

    pub async fn try_measure(&mut self) -> Result<D::Measurement, Error> {
        let res = self.driver.measure().await.map_err(Error::Running);
        if res.is_err() {
            self.driver.reset().await.map_err(Error::Setup)?;
        }
        res
    }
}

impl<'a> ResettableDriver for mhzx::MHZ<ConcreteTx<'a>, ConcreteRx<'a>> {
    type Measurement = f32;

    async fn reset(&mut self) -> Result<Self, SensorError> {
        self.
    }

    async fn measure(&mut self) -> Result<Self::Measurement, SensorError> {
        todo!()
    }
}
