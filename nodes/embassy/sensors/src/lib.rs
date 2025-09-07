#![no_std]

mod drivers;
pub mod buttons;
pub mod errors;
pub mod measurements;

pub use drivers::I2cWrapper;
pub use drivers::{
    Bme680Driver, Max44Driver, MhzDriver, Nau7802Driver, Nau7802DriverBlocking,
    Sht31Driver, Sps30Driver, SPS30_DRIVER_BUF_SIZE, SPS30_UART_BUF_SIZE,
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
