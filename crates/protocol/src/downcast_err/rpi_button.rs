use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
/// The gpio_cdev::Error is not constant size and impossible to inspect. The
/// alternative would be to send the error as a string. The error struct must
/// have a statically determined maximum size to allow encoding/decoding on
/// embedded systems. Thus sending a string is also not an option
pub enum RpiButtonError {
    #[cfg_attr(
        feature = "thiserror",
        error("Error interfacing with Linux GPIO interface")
    )]
    GpioInterfaceError,
    #[cfg_attr(
        feature = "thiserror",
        error("Error interfacing with Linux GPIO interface")
    )]
    GpioChipNotFound,
    #[cfg_attr(
        feature = "thiserror",
        error("Error interfacing with Linux GPIO interface")
    )]
    ListingChips,
    #[cfg_attr(
        feature = "thiserror",
        error("Error interfacing with Linux GPIO interface")
    )]
    GettingLine(u32),
    #[cfg_attr(
        feature = "thiserror",
        error("Error interfacing with Linux GPIO interface")
    )]
    GetEventValue,
}
