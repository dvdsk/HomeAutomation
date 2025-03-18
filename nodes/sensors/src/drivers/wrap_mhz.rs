use mhzx::MHZ;
use protocol::Device;

use crate::errors::SensorError;
use crate::{Driver, Error};

use super::{ConcreteRx, ConcreteTx};

pub struct MhzDriver<'a> {
    driver: MHZ<ConcreteTx<'a>, ConcreteRx<'a>>,
    device: protocol::Device,
}

impl<'a> MhzDriver<'a> {
    pub fn new(
        uart_tx: ConcreteTx<'a>,
        uart_rx: ConcreteRx<'a>,
        device: Device,
    ) -> Self {
        Self {
            driver: MHZ::from_tx_rx(uart_tx, uart_rx),
            device,
        }
    }
}

impl<'a> Driver for MhzDriver<'a> {
    type Measurement = mhzx::Measurement;
    type Affector = ();

    #[inline(always)]
    async fn try_measure(&mut self) -> Result<Self::Measurement, Error> {
        self.driver
            .read_co2()
            .await
            .map_err(SensorError::Mhz14)
            .map_err(Error::Running)
    }

    #[inline(always)]
    fn device(&self) -> protocol::Device {
        self.device.clone()
    }
}
