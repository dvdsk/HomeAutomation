use embassy_stm32::i2c::{Config, Error};
use embassy_stm32::mode::Async;
use embassy_stm32::time::Hertz;
use embedded_hal::i2c::ErrorType;

pub struct I2c<'d> {
    inner: embassy_stm32::i2c::I2c<'d, Async>,
    freq: Hertz,
    config: Config,
}

// impl<'d> From<embassy_stm32::i2c::I2c<'d, Async>> for I2c<'d> {
//     fn from(value: embassy_stm32::i2c::I2c<'d, Async>) -> Self {
//         Self { inner: value }
//     }
// }

impl<'d> I2c<'d> {
    pub fn new(i2c: embassy_stm32::i2c::I2c<'d, Async>, freq: Hertz, config: Config) -> Self {
        Self {
            inner: i2c,
            freq,
            config,
        }
    }
}

impl ErrorType for I2c<'_> {
    type Error = Error;
}

impl embedded_hal_async::i2c::I2c for I2c<'_> {
    async fn read(&mut self, address: u8, read: &mut [u8]) -> Result<(), Self::Error> {
        let res = self.inner.read(address, read).await;
        if let Err(Error::Arbitration) = res {
            self.inner.clear_error();
            self.inner.reset(self.freq, self.config)
        }
        return res;
    }

    async fn write(&mut self, address: u8, write: &[u8]) -> Result<(), Self::Error> {
        let res = self.inner.write(address, write).await;
        if let Err(Error::Arbitration) = res {
            self.inner.clear_error();
        }
        return res;
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        let res = self.inner.write_read(address, write, read).await;
        if let Err(Error::Arbitration) = res {
            self.inner.clear_error();
        }
        return res;
    }

    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        let res = self.inner.transaction(address, operations).await;
        if let Err(Error::Arbitration) = res {
            self.inner.clear_error();
        }
        return res;
    }
}
