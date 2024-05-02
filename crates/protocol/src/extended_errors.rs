use bosch_bme680::BmeError;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, defmt::Format, Serialize, Deserialize, MaxSize)]
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

pub trait ConcreteErrorType {
    type Concrete;
    fn strip_generics(self) -> Self::Concrete;
}

impl<E> ConcreteErrorType for BmeError<E>
where
    E: Into<I2cError> + core::fmt::Debug,
{
    type Concrete = BmeError<I2cError>;
    fn strip_generics(self) -> Self::Concrete {
        match self {
            BmeError::WriteError(i2c) => BmeError::WriteError(i2c.into()),
            BmeError::WriteReadError(i2c) => BmeError::WriteReadError(i2c.into()),
            BmeError::UnexpectedChipId(id) => BmeError::UnexpectedChipId(id),
            BmeError::MeasuringTimeOut => BmeError::MeasuringTimeOut,
        }
    }
}

impl<E> ConcreteErrorType for max44009::Error<E>
where
    E: Into<I2cError> + core::fmt::Debug + defmt::Format,
{
    type Concrete = max44009::Error<I2cError>;
    fn strip_generics(self) -> Self::Concrete {
        match self {
            max44009::Error::I2C(i2c) => max44009::Error::I2C(i2c.into()),
            max44009::Error::OperationNotAvailable => max44009::Error::OperationNotAvailable,
        }
    }
}
