/// The gpio_cdev::Error is not constant size. The alternative would be to send
/// the error as a fixed size string. The error struct must have a statically
/// determined maximum size to allow encoding/decoding on embedded systems. Thus
/// sending a string is also not an option
///
/// We could add a second message type with variable message size but we would
/// still need to add serde support to the origonal error
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
pub enum CdevError {
    /// An operation cannot be performed due to a limitation in the ABI version being used.
    #[cfg_attr(
        feature = "thiserror",
        error(
            "An operation cannot be performed due to a limitation in the ABI version being used."
        )
    )]
    AbiLimitation,
    /// Problem accessing GPIO chip character devices
    #[cfg_attr(
        feature = "thiserror",
        error("Problem accessing GPIO chip character devices")
    )]
    GpioChip,
    /// An error returned when there is a problem with an argument.
    #[cfg_attr(
        feature = "thiserror",
        error("An error returned when there is a problem with an argument.")
    )]
    InvalidArgument,
    /// No gpiochips are available to the user.
    #[cfg_attr(
        feature = "thiserror",
        error("No gpiochips are available to the user.")
    )]
    NoGpioChips,
    /// Returned when the strict mode of find_named_lines finds multiple lines with the same name.
    #[cfg_attr(feature = "thiserror", error("Returned when the strict mode of find_named_lines finds multiple lines with the same name."))]
    NonuniqueLineName,
    /// An error returned from an underlying os call.
    #[cfg_attr(
        feature = "thiserror",
        error("An error returned from an underlying os call.")
    )]
    Os,
    /// An error returned from an underlying uAPI call.
    #[cfg_attr(
        feature = "thiserror",
        error("An error returned from an underlying uAPI call.")
    )]
    Uapi,
    /// The response to a uAPI command contained unexpected content.
    #[cfg_attr(
        feature = "thiserror",
        error("The response to a uAPI command contained unexpected content.")
    )]
    UnexpectedResponse,
    /// The kernel or build does not support the requested uAPI ABI version.
    #[cfg_attr(
        feature = "thiserror",
        error("The kernel or build does not support the requested uAPI ABI version.")
    )]
    UnsupportedAbi,
    /// The kernel has no support for any uAPI ABI version.
    #[cfg_attr(
        feature = "thiserror",
        error("The kernel has no support for any uAPI ABI version.")
    )]
    NoAbiSupport,
}

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
pub enum GpioError {
    /// Requests can only contain a single requested line.
    #[cfg_attr(
        feature = "thiserror",
        error("Requests can only contain a single requested line.")
    )]
    MultipleLinesRequested,

    /// InputPins must be in input mode.
    #[cfg_attr(feature = "thiserror", error("InputPins must be in input mode."))]
    RequiresInputMode,

    /// OutputPins must be in output mode.
    #[cfg_attr(feature = "thiserror", error("OutputPins must be in output mode."))]
    RequiresOutputMode,

    /// Cannot find named line.
    #[cfg_attr(feature = "thiserror", error("Cannot find named line."))]
    UnfoundLine,

    /// An error returned from an underlying gpiocdev call.
    #[cfg_attr(
        feature = "thiserror",
        error("An error returned from an underlying gpiocdev call.")
    )]
    Cdev(CdevError),
}

impl From<gpiocdev_embedded_hal::Error> for GpioError {
    fn from(value: gpiocdev_embedded_hal::Error) -> Self {
        use gpiocdev_embedded_hal::Error;
        match value {
            Error::MultipleLinesRequested => GpioError::MultipleLinesRequested,
            Error::RequiresInputMode => GpioError::RequiresInputMode,
            Error::RequiresOutputMode => GpioError::RequiresOutputMode,
            Error::UnfoundLine(_) => GpioError::UnfoundLine,
            Error::Cdev(e) => GpioError::Cdev(e.into()),
        }
    }
}

impl From<gpiocdev::Error> for CdevError {
    fn from(value: gpiocdev::Error) -> Self {
        use gpiocdev::Error;
        match value {
            Error::AbiLimitation(_, _) => CdevError::AbiLimitation,
            Error::GpioChip(_, _) => CdevError::GpioChip,
            Error::InvalidArgument(_) => CdevError::InvalidArgument,
            Error::NoGpioChips() => CdevError::NoGpioChips,
            Error::NonuniqueLineName(_) => CdevError::NonuniqueLineName,
            Error::Os(_) => CdevError::Os,
            Error::Uapi(_, _) => CdevError::Uapi,
            Error::UnexpectedResponse(_) => CdevError::UnexpectedResponse,
            Error::UnsupportedAbi(_, _) => CdevError::UnsupportedAbi,
            Error::NoAbiSupport() => CdevError::NoAbiSupport,
        }
    }
}
