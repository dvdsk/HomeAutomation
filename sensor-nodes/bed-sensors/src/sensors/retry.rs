use bosch_bme680::{BmeError, Configuration, DeviceAddress, MeasurementData};
use embassy_embedded_hal::shared_bus::{self, I2cDeviceError};
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Delay;

// use super::concrete_types::ConcreteRx as Rx;
use super::concrete_types::ConcreteSharedI2c;
// use super::concrete_types::ConcreteTx as Tx;
use crate::error_cache::{Error, SensorError};

enum Bme680<'a> {
    Ready {
        bme: bosch_bme680::Bme680<ConcreteSharedI2c<'a>, Delay>,
        i2c: &'a Mutex<NoopRawMutex, I2c<'static, Async>>,
    },
    Uninit {
        i2c: &'a Mutex<NoopRawMutex, I2c<'static, Async>>,
    },
}

impl<'a> Bme680<'a> {
    pub fn new(i2c: &'a Mutex<NoopRawMutex, I2c<'static, Async>>) -> Self {
        Self::Uninit { i2c }
    }

    /// cancel safe, any cancelation will end us in the Uninit state
    /// which is where we want to be after stopping in the middle of
    /// a transaction
    pub async fn try_measure(&mut self) -> Result<MeasurementData, Error> {
        let i2c = *match self {
            Bme680::Ready { i2c, .. } => i2c,
            Bme680::Uninit { i2c } => i2c,
        };

        let mut owned_self = Self::Uninit { i2c };
        core::mem::swap(&mut owned_self, self);
        let (mut new_self, res) = owned_self.advance_state().await;
        core::mem::swap(&mut new_self, self);
        res
    }

    async fn advance_state(mut self) -> (Self, Result<MeasurementData, Error>) {
        match self {
            Bme680::Ready { ref mut bme, i2c } => match bme.measure().await {
                Ok(val) => return (self, Ok(val)),
                Err(err) => {
                    let new_self = Self::Uninit { i2c };
                    let err = Error::Running(SensorError::Bme680(err));
                    (new_self, Err(err))
                }
            },
            Bme680::Uninit { i2c } => {
                let shared_i2c = shared_bus::asynch::i2c::I2cDevice::new(i2c);
                match bosch_bme680::Bme680::new(
                    shared_i2c,
                    DeviceAddress::Secondary,
                    Delay,
                    &Configuration::default(),
                    21,
                )
                .await
                {
                    Ok(mut bme) => {
                        let val = match bme.measure().await {
                            Ok(val) => val,
                            Err(err) => {
                                let err = Error::Running(SensorError::Bme680(err));
                                return (self, Err(err));
                            }
                        };
                        let new_self = Self::Ready { bme, i2c };
                        (new_self, Ok(val))
                    }
                    Err(err) => {
                        let err = Error::Setup(SensorError::Bme680(err));
                        (self, Err(err))
                    }
                }
            }
        }
    }
}
