use embassy_embedded_hal::shared_bus::I2cDeviceError;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;

pub struct I2cWrapper<T>(pub Mutex<NoopRawMutex, T>);

/// Make sure the error type is the same as when
/// embassy_embedded_hal::shared_bus async is used. We do so by wrapping
/// underlying errors in [`I2cDevice`].
///
/// [`I2cDevice`](embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice)
impl<T: embedded_hal::i2c::ErrorType> embedded_hal_async::i2c::ErrorType
    for &I2cWrapper<T>
{
    type Error = I2cDeviceError<<T as embedded_hal::i2c::ErrorType>::Error>;
}

impl<T: embedded_hal::i2c::I2c> embedded_hal_async::i2c::I2c
    for &I2cWrapper<T>
{
    async fn transaction(
        &mut self,
        address: u8,
        operations: &mut [embedded_hal_async::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        self.0
            .lock()
            .await
            .transaction(address, operations)
            .map_err(I2cDeviceError::I2c)
    }

    async fn read(
        &mut self,
        address: u8,
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.0
            .lock()
            .await
            .read(address, read)
            .map_err(I2cDeviceError::I2c)
    }

    async fn write(
        &mut self,
        address: u8,
        write: &[u8],
    ) -> Result<(), Self::Error> {
        self.0
            .lock()
            .await
            .write(address, write)
            .map_err(I2cDeviceError::I2c)
    }

    async fn write_read(
        &mut self,
        address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.0
            .lock()
            .await
            .write_read(address, write, read)
            .map_err(I2cDeviceError::I2c)
    }
}
