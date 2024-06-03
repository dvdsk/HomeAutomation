use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
pub enum I2cError {
    #[cfg_attr(feature = "thiserror", error("Bus error"))]
    Bus,
    #[cfg_attr(feature = "thiserror", error("Arbitration lost"))]
    Arbitration,
    #[cfg_attr(
        feature = "thiserror",
        error("ACK not received (either to the address or to a data byte)")
    )]
    Nack,
    #[cfg_attr(feature = "thiserror", error("Timeout"))]
    Timeout,
    #[cfg_attr(feature = "thiserror", error("CRC error"))]
    Crc,
    #[cfg_attr(feature = "thiserror", error("Overrun error"))]
    Overrun,
    #[cfg_attr(feature = "thiserror", error("Zero-length transfers are not allowed"))]
    ZeroLengthTransfer,
    #[cfg_attr(
        feature = "thiserror",
        error("Configuration of the inner I2C bus failed.")
    )]
    Config,
}

impl From<embassy_embedded_hal::shared_bus::I2cDeviceError<embassy_stm32::i2c::Error>>
    for I2cError
{
    fn from(
        value: embassy_embedded_hal::shared_bus::I2cDeviceError<embassy_stm32::i2c::Error>,
    ) -> Self {
        use embassy_embedded_hal::shared_bus::I2cDeviceError;
        use embassy_stm32::i2c::Error;

        match value {
            I2cDeviceError::I2c(Error::Bus) => Self::Bus,
            I2cDeviceError::I2c(Error::Arbitration) => Self::Arbitration,
            I2cDeviceError::I2c(Error::Nack) => Self::Nack,
            I2cDeviceError::I2c(Error::Timeout) => Self::Timeout,
            I2cDeviceError::I2c(Error::Crc) => Self::Crc,
            I2cDeviceError::I2c(Error::Overrun) => Self::Overrun,
            I2cDeviceError::I2c(Error::ZeroLengthTransfer) => Self::ZeroLengthTransfer,
            I2cDeviceError::Config => Self::Config,
        }
    }
}

#[derive(Clone, Copy, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
// linux error codes should be stable across different linux kernels
pub enum LinuxI2cError {
    FailedLibcCall(i32),
    Io(Option<i32>),
}

#[cfg(feature = "thiserror")]
impl core::fmt::Display for LinuxI2cError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LinuxI2cError::FailedLibcCall(failed_libc_call) => {
                let errno = nix::errno::Errno::from_raw(*failed_libc_call);
                write!(f, "Error calling I2C interface: {errno}")
            }
            LinuxI2cError::Io(Some(raw_os_error)) => {
                let io_error = std::io::Error::from_raw_os_error(*raw_os_error);
                write!(f, "Io error using I2C interface: {io_error}, more details in logs on sensing device")
            }
            LinuxI2cError::Io(None) => {
                write!(
                    f,
                    "Unknown error using I2C interface, check logs on sensing device"
                )
            }
        }
    }
}

#[cfg(alloc)]
impl From<linux_embedded_hal::I2CError> for LinuxI2cError {
    fn from(value: linux_embedded_hal::I2CError) -> Self {
        value.inner().into()
    }
}

#[cfg(alloc)]
impl From<&linux_embedded_hal::i2cdev::linux::LinuxI2CError> for LinuxI2cError {
    fn from(value: &linux_embedded_hal::i2cdev::linux::LinuxI2CError) -> Self {
        use linux_embedded_hal::i2cdev::linux::LinuxI2CError;

        match value {
            LinuxI2CError::Errno(failing_libc_call) => {
                LinuxI2cError::FailedLibcCall(*failing_libc_call)
            }
            LinuxI2CError::Io(io_error) => LinuxI2cError::Io(io_error.raw_os_error()),
        }
    }
}
