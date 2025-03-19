use embassy_embedded_hal::shared_bus::I2cDeviceError;
use protocol::make_error_string;

pub type I2cError = I2cDeviceError<embassy_stm32::i2c::Error>;
pub type UartError = embassy_stm32::usart::Error;

#[derive(defmt::Format, PartialEq, Eq, Clone)]
pub struct PressTooLong {
    pub button: &'static str,
}

impl core::fmt::Debug for PressTooLong {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Button {}, pressed too long", self.button))
    }
}

#[derive(defmt::Format, PartialEq, Eq, Clone)]
pub enum SensorError {
    Mhz14(mhzx::Error<UartError, UartError>),
    Sps30(sps30_async::Error<UartError, UartError>),
    Sht31(sht31::SHTError),
    Bme680(bosch_bme680::BmeError<I2cError>),
    Max44(max44009::Error<I2cError>),
    /// Nodes with only one Nau7802 should use left,
    /// it is what the driver returns by default
    Nau7802Left(nau7802_async::Error<I2cError>),
    Nau7802Right(nau7802_async::Error<I2cError>),
    Button(PressTooLong),
}

impl SensorError {
    /// ugly hack to allow users of sensors to get a different error for two
    /// identical devices by calling into_right on the error the default error
    /// is the 'left device'. Works only if there is a right device. Needs
    /// the From<Error> for protocol::_::_Error to map right devices to right
    /// devices
    fn into_right(self) -> Self {
        match self {
            Self::Nau7802Left(e) => Self::Nau7802Right(e),
            ref _other => self,
        }
    }
}

/// This error type is as small as possible. It is meant for
/// devices to deduplicate errors that happen rapidly and only
/// send such errors every once in a while to the host
#[derive(defmt::Format, PartialEq, Eq, Clone)]
pub enum Error {
    Running(SensorError),
    Setup(SensorError),
    Timeout(protocol::Device),
}

impl Error {
    /// ugly hack to allow users of sensors to get a different error for two
    /// identical devices by calling into_right on the error the default error
    /// is the 'left device'. Works only if there is a right device. Needs
    /// the From<Error> for protocol::_::_Error to map right devices to right
    /// devices
    pub fn into_right(self) -> Self {
        match self {
            Self::Running(sensor_error) => {
                Self::Running(sensor_error.into_right())
            }
            Self::Setup(sensor_error) => {
                Self::Setup(sensor_error.into_right())
            }
            Self::Timeout(ref _device) => self,
        }
    }
}

impl From<SensorError> for protocol::small_bedroom::bed::SensorError {
    fn from(val: SensorError) -> Self {
        use protocol::small_bedroom::bed;
        match val {
            SensorError::Mhz14(e) => {
                bed::SensorError::Mhz14(make_error_string(e))
            }
            SensorError::Sps30(e) => {
                bed::SensorError::Sps30(make_error_string(e))
            }
            SensorError::Sht31(e) => {
                bed::SensorError::Sht31(make_error_string(e))
            }
            SensorError::Bme680(e) => {
                bed::SensorError::Bme680(make_error_string(e))
            }
            SensorError::Max44(e) => {
                bed::SensorError::Max44(make_error_string(e))
            }
            SensorError::Nau7802Left(e) => {
                bed::SensorError::Nau7802(make_error_string(e))
            }
            SensorError::Button(e) => {
                bed::SensorError::Button(make_error_string(e))
            }
            SensorError::Nau7802Right(_) => {
                unreachable!("Not installed for small_bedroom bed sensor")
            }
        }
    }
}

impl From<Error> for protocol::small_bedroom::bed::Error {
    fn from(val: Error) -> Self {
        use protocol::small_bedroom::bed;
        match val {
            Error::Running(e) => bed::Error::Running(e.into()),
            Error::Setup(e) => bed::Error::Setup(e.into()),
            Error::Timeout(protocol::Device::SmallBedroom(
                protocol::small_bedroom::Device::Bed(dev),
            )) => bed::Error::Timeout(dev),
            Error::Timeout(incorrect_dev) => defmt::unreachable!(
                "Should only contain small_bedroom::bed devices, got {} which \
                is not part of small_bedroom::bed::Device",
                incorrect_dev
            ),
        }
    }
}

impl From<SensorError> for protocol::large_bedroom::bed::SensorError {
    fn from(val: SensorError) -> Self {
        use protocol::large_bedroom::bed;
        match val {
            SensorError::Mhz14(e) => {
                bed::SensorError::Mhz14(make_error_string(e))
            }
            SensorError::Sps30(e) => {
                bed::SensorError::Sps30(make_error_string(e))
            }
            SensorError::Sht31(e) => {
                bed::SensorError::Sht31(make_error_string(e))
            }
            SensorError::Bme680(e) => {
                bed::SensorError::Bme680(make_error_string(e))
            }
            SensorError::Max44(e) => {
                bed::SensorError::Max44(make_error_string(e))
            }
            SensorError::Nau7802Left(e) => {
                bed::SensorError::Nau7802Left(make_error_string(e))
            }
            SensorError::Nau7802Right(e) => {
                bed::SensorError::Nau7802Right(make_error_string(e))
            }
            SensorError::Button(e) => {
                bed::SensorError::Button(make_error_string(e))
            }
        }
    }
}

impl From<Error> for protocol::large_bedroom::bed::Error {
    fn from(val: Error) -> Self {
        use protocol::large_bedroom::bed;
        match val {
            Error::Running(e) => bed::Error::Running(e.into()),
            Error::Setup(e) => bed::Error::Setup(e.into()),
            Error::Timeout(protocol::Device::LargeBedroom(
                protocol::large_bedroom::Device::Bed(dev),
            )) => bed::Error::Timeout(dev),
            Error::Timeout(incorrect_dev) => defmt::unreachable!(
                "Should only contain small_bedroom::bed devices, got {} which \
                is not part of small_bedroom::bed::Device",
                incorrect_dev
            ),
        }
    }
}
