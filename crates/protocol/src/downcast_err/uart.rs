use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, defmt::Format, Serialize, Deserialize, MaxSize, Eq, PartialEq)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
pub enum Error {
    #[cfg_attr(feature = "thiserror", error("Framing error"))]
    Framing,
    #[cfg_attr(feature = "thiserror", error("Noise error"))]
    Noise,
    #[cfg_attr(feature = "thiserror", error("RX buffer overrun"))]
    Overrun,
    #[cfg_attr(feature = "thiserror", error("Parity check error"))]
    Parity,
    #[cfg_attr(feature = "thiserror", error("Buffer too large for DMA"))]
    BufferTooLong,

    // Author note: I strongly dislike non-exhaustive for this reason. I want to
    // know I handle every error case at compile time that is why we have a type
    // system! Now I can update the code and run into an unhandled error years
    // after the update. The opsession with never increasing major versions that
    // has caused this spread of non-exhaustive needs to stop.
    #[cfg_attr(feature = "thiserror", error("A new error was added upstream unknown to the protocol and it occurred, please update the From<embassy usart> for UsartError implementation"))]
    NewUnhandledError,
}

impl From<embassy_stm32::usart::Error> for Error {
    fn from(err: embassy_stm32::usart::Error) -> Self {
        use embassy_stm32::usart::Error;

        match err {
            Error::Framing => Self::Framing,
            Error::Noise => Self::Noise,
            Error::Overrun => Self::Overrun,
            Error::Parity => Self::Parity,
            Error::BufferTooLong => Self::BufferTooLong,
            _ => Self::NewUnhandledError,
        }
    }
}
