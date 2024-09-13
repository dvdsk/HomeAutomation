use embassy_time::Delay;
use sps30_async::Sps30;

use crate::error_cache::{Error, SensorError};
use crate::sensors::concrete_types::{ConcreteRx, ConcreteTx};
use crate::sensors::SPS30_UART_BUF_SIZE;

use super::Driver;

pub struct Sps30Driver<'a> {
    is_init: bool,
    driver: Sps30<SPS30_UART_BUF_SIZE, ConcreteTx<'a>, ConcreteRx<'a>, Delay>,
}

impl<'a> Sps30Driver<'a> {
    pub fn init(tx: ConcreteTx<'a>, rx: ConcreteRx<'a>) -> Self {
        let driver = Sps30::from_tx_rx_uninit(tx, rx, Delay);
        Self {
            is_init: false,
            driver,
        }
    }

    async fn measure(&mut self) -> Result<sps30_async::Measurement, Error> {
        if self.is_init {
            return self
                .driver
                .read_measurement()
                .await
                .map_err(SensorError::Sps30)
                .map_err(Error::Running);
        }

        self.driver
            .reset()
            .await
            .map_err(SensorError::Sps30)
            .map_err(Error::Setup)?;
        self.driver
            .start_measurement()
            .await
            .map_err(SensorError::Sps30)
            .map_err(Error::Setup)?;

        self.is_init = true;
        self.driver
            .read_measurement()
            .await
            .map_err(SensorError::Sps30)
            .map_err(Error::Running)
    }
}

impl<'a> Driver for Sps30Driver<'a> {
    type Measurement = sps30_async::Measurement;
    type Affector = ();

    async fn try_measure(&mut self) -> Result<Self::Measurement, crate::error_cache::Error> {
        self.measure().await
    }

    async fn affect(&mut self, _: Self::Affector) -> Result<(), crate::error_cache::Error> {
        self.driver
            .start_fan_cleaning()
            .await
            .map_err(SensorError::Sps30)
            .map_err(Error::Running)?;
        // Fan cleaning takes 10 seconds, make sure its done before 
        // allowing the next reading
        embassy_time::Timer::after(embassy_time::Duration::from_secs(11)).await;
        Ok(())
    }

    fn device(&self) -> protocol::large_bedroom::bed::Device {
        protocol::large_bedroom::bed::Device::Sps30
    }
}
