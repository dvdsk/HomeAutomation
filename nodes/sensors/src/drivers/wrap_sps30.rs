use embassy_time::Delay;
use sps30_async::Sps30;

use crate::errors::{Error, SensorError};
use crate::Driver;

use super::{ConcreteRx, ConcreteTx};

pub const SPS30_UART_BUF_SIZE: usize = 150;
pub const SPS30_DRIVER_BUF_SIZE: usize = 2 * SPS30_UART_BUF_SIZE;

/// reset the device when an error occurs
pub struct Sps30Driver<'a> {
    is_init: bool,
    driver: Sps30<SPS30_DRIVER_BUF_SIZE, ConcreteTx<'a>, ConcreteRx<'a>, Delay>,
    device: protocol::Device,
}

impl<'a> Sps30Driver<'a> {
    pub fn init(
        tx: ConcreteTx<'a>,
        rx: ConcreteRx<'a>,
        device: protocol::Device,
    ) -> Self {
        let driver = Sps30::from_tx_rx_uninit(tx, rx, Delay);
        Self {
            is_init: false,
            driver,
            device,
        }
    }

    async fn measure(&mut self) -> Result<sps30_async::Measurement, Error> {
        if self.is_init {
            let res = self
                .driver
                .read_measurement()
                .await
                .map_err(SensorError::Sps30)
                .map_err(Error::Running);
            self.is_init = res.is_ok();
            return res;
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

        let res = self
            .driver
            .read_measurement()
            .await
            .map_err(SensorError::Sps30)
            .map_err(Error::Running);

        self.is_init = res.is_ok();
        res
    }
}

impl<'a> Driver for Sps30Driver<'a> {
    type Measurement = sps30_async::Measurement;
    type Affector = ();

    async fn try_measure(&mut self) -> Result<Self::Measurement, Error> {
        self.measure().await
    }

    async fn affect(&mut self, _: Self::Affector) -> Result<(), Error> {
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

    fn device(&self) -> protocol::Device {
        self.device.clone()
    }
}
