#![cfg_attr(not(feature = "alloc"), no_std)]

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

pub mod large_bedroom;
pub mod small_bedroom;

pub mod downcast_err;

/// turns an enum of empty variants into one with [`Press`](crate::Press) inside each variant also
/// adds a method `press(&self)` that returns an instance of [`Press`](crate::Press). It can be
/// used to quickly find out if a button event is a long or short press.
///
/// Example:
///
/// button_enum! {
///     DeskButton {
///         OneOfFour,
///         TwoOfFour,
///     }
/// }
///  
macro_rules! button_enum {
    (
        $(#[$outer:meta])*
        $name:ident {$($variant:ident,)*}) => {
        #[derive(strum::VariantNames)]
        #[derive(
            Clone,
			Copy,
			Debug,
			serde::Serialize,
			serde::Deserialize,
            defmt::Format,
            postcard::experimental::max_size::MaxSize
        )]
        $(#[$outer])* // docs
        #[repr(u8)]
        pub enum $name {
            $($variant(crate::Press),)*
        }

        impl $name {
            pub fn press(&self) -> crate::Press {
                match self {
                    $(Self::$variant(d) => *d,)*
                }
            }
        }

        impl Into<f32> for $name {
            fn into(self) -> f32 {
                match self {
                    $(Self::$variant(p) if p.is_long() => 2.0,)*
                    $(Self::$variant(_) => 1.0,)*
                }
            }
        }
    };
}
pub(crate) use button_enum;

// Duration of the press in milliseconds
#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    postcard::experimental::max_size::MaxSize,
    defmt::Format,
)]
pub struct Press(pub u16);

impl Press {
    pub fn is_long(&self) -> bool {
        if self.0 > 300 {
            true
        } else {
            false
        }
    }
}

#[derive(strum::VariantNames, Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize)]
pub enum Reading {
    LargeBedroom(large_bedroom::Reading),
    // SmallBedroom(small_bedroom::Reading),
    Test,
}

#[derive(strum::VariantNames, Clone, Debug, defmt::Format, Serialize, Deserialize, MaxSize)]
pub enum Error {
    LargeBedroom(large_bedroom::Error),
}

impl Reading {
    pub fn version() -> u8 {
        0u8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorMessage<const MAX_ITEMS: usize> {
    pub values: heapless::Vec<Result<Reading, Error>, MAX_ITEMS>,
    pub version: u8,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
#[cfg_attr(feature = "thiserror", error("Could not decode SensorMessage: {0}"))]
pub struct DecodeError(pub postcard::Error);

impl<const MAX_ITEMS: usize> SensorMessage<MAX_ITEMS> {
    /// the 2x is the max overhead from COBS encoding the encoded data
    /// +2 is for the version
    /// +4 covers the length of the heapless list
    pub const ENCODED_SIZE: usize = 2 * (MAX_ITEMS * Reading::POSTCARD_MAX_SIZE + 2 + 4);

    pub fn new() -> Self {
        Self {
            values: heapless::Vec::new(),
            version: 0,
        }
    }

    pub fn space_left(&self) -> bool {
        self.values.len() < self.values.capacity()
    }

    #[cfg(feature = "alloc")]
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec_cobs(self).expect("Encoding should not fail")
    }

    /// Buffer should be at least Self::ENCODED_SIZE long. The returned slice contains
    /// the serialized data. It can be shorter then the input buffer.
    pub fn encode_slice<'a>(&self, buf: &'a mut [u8]) -> &'a mut [u8] {
        postcard::to_slice_cobs(self, buf).expect("Encoding should not fail")
    }

    pub fn decode(mut bytes: impl AsMut<[u8]>) -> Result<Self, DecodeError> {
        postcard::from_bytes_cobs(bytes.as_mut()).map_err(DecodeError)
    }

    pub fn version(&self) -> u8 {
        self.version
    }
}
