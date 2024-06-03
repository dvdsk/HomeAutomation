mod i2c;
// #[cfg(feature = "alloc")]
mod rpi_button;
mod uart;

use bosch_bme680::BmeError;
pub use i2c::{I2cError, LinuxI2cError};
// #[cfg(feature = "alloc")]
pub use rpi_button::GpioError;
pub use uart::Error as UartError;

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

impl<E1, E2> ConcreteErrorType for mhzx::Error<E1, E2>
where
    E1: Into<UartError> + core::fmt::Debug + defmt::Format,
    E2: Into<UartError> + core::fmt::Debug + defmt::Format,
{
    type Concrete = mhzx::Error<UartError, UartError>;
    fn strip_generics(self) -> Self::Concrete {
        match self {
            mhzx::Error::InvalidChecksum => Self::Concrete::InvalidChecksum,
            mhzx::Error::InvalidPacket => Self::Concrete::InvalidPacket,
            mhzx::Error::WritingToUart(e) => Self::Concrete::WritingToUart(e.into()),
            mhzx::Error::FlushingUart(e) => Self::Concrete::FlushingUart(e.into()),
            mhzx::Error::ReadingEOF => Self::Concrete::ReadingEOF,
            mhzx::Error::Reading(e) => Self::Concrete::Reading(e.into()),
        }
    }
}

impl<E1, E2> ConcreteErrorType for sps30_async::Error<E1, E2>
where
    E1: Into<UartError> + core::fmt::Debug + defmt::Format,
    E2: Into<UartError> + core::fmt::Debug + defmt::Format,
{
    type Concrete = sps30_async::Error<UartError, UartError>;
    fn strip_generics(self) -> Self::Concrete {
        match self {
            sps30_async::Error::SerialR(e) => sps30_async::Error::SerialR(e.into()),
            sps30_async::Error::SerialW(e) => sps30_async::Error::SerialW(e.into()),
            sps30_async::Error::SHDLC(e) => sps30_async::Error::SHDLC(e),
            sps30_async::Error::InvalidFrame => sps30_async::Error::InvalidFrame,
            sps30_async::Error::EmptyResult => sps30_async::Error::EmptyResult,
            sps30_async::Error::ChecksumFailed => sps30_async::Error::ChecksumFailed,
            sps30_async::Error::InvalidResponse => sps30_async::Error::InvalidResponse,
            sps30_async::Error::DeviceError(e) => sps30_async::Error::DeviceError(e),
            sps30_async::Error::MeasurementDataTooShort => {
                sps30_async::Error::MeasurementDataTooShort
            }
            sps30_async::Error::CleaningIntervalDataTooShort => {
                sps30_async::Error::CleaningIntervalDataTooShort
            }
            sps30_async::Error::SerialInvalidUtf8 => sps30_async::Error::SerialInvalidUtf8,
            sps30_async::Error::ReadingEOF => sps30_async::Error::ReadingEOF,
            sps30_async::Error::FrameTooLarge => sps30_async::Error::FrameTooLarge,
        }
    }
}

impl<E> ConcreteErrorType for bme280::Error<E>
where
    E: Into<LinuxI2cError> + core::fmt::Debug,
{
    type Concrete = bme280::Error<LinuxI2cError>;
    fn strip_generics(self) -> Self::Concrete {
        match self {
            bme280::Error::CompensationFailed => bme280::Error::CompensationFailed,
            bme280::Error::Bus(e) => bme280::Error::Bus(e.into()),
            bme280::Error::InvalidData => bme280::Error::InvalidData,
            bme280::Error::NoCalibrationData => bme280::Error::NoCalibrationData,
            bme280::Error::UnsupportedChip => bme280::Error::UnsupportedChip,
            bme280::Error::Delay => bme280::Error::Delay,
        }
    }
}
