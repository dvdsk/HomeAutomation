#![no_std]

mod drivers;
pub mod measurements;
pub mod errors;

pub use drivers::{
    Bme680Driver, Max44Driver, Nau7802Driver, Sps30Driver, Sht31Driver, MhzDriver,
    SPS30_DRIVER_BUF_SIZE, SPS30_UART_BUF_SIZE,
};

pub use errors::Error;
use errors::SensorError;

#[allow(async_fn_in_trait)]
pub trait Driver {
    type Measurement;
    type Affector;

    async fn try_measure(&mut self) -> Result<Self::Measurement, Error>;
    /// For example calibrate, run sensor cleaning etc
    async fn affect(&mut self, _: Self::Affector) -> Result<(), Error> {
        Ok(())
    }
    fn device(&self) -> protocol::Device;
}

/// A driver that can be re-initialized from copy-able parts
#[allow(async_fn_in_trait)]
pub trait ReInitableDriver: Sized {
    type Parts: Clone;
    type Measurement;

    async fn init(parts: Self::Parts) -> Result<Self, SensorError>;
    async fn measure(&mut self) -> Result<Self::Measurement, SensorError>;
}
