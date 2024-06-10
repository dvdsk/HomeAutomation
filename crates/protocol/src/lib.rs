#![cfg_attr(not(feature = "alloc"), no_std)]

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

pub mod large_bedroom;
pub mod small_bedroom;

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
        /// SAFETY: must be repr(u8) or id fn will create undefined behaviour
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

        #[cfg(feature = "alloc")]
        impl crate::Tomato for $name {
            fn inner<'a>(&'a self) -> crate::TomatoItem<'a> {
                crate::TomatoItem::Leaf((*self).into())
            }
            fn name(&self) -> String {
                let dbg_repr = format!("{:?}", self);
                dbg_repr
                    .split_once('(')
                    .map(|(name, _)| name)
                    .unwrap_or("-")
                    .to_string()
            }
            fn id(&self) -> crate::TomatoId {
                // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a
                // `repr(C)` `union` between `repr(C)` structs, each of which
                // has the `u8` discriminant as its first field, so we can read
                // the discriminant without offsetting the pointer.
                let discriminant = unsafe { *<*const _>::from(self).cast::<u8>() };
                discriminant as crate::TomatoId
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

#[cfg(feature = "alloc")]
type TomatoId = u8;
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub enum TomatoItem<'a> {
    Leaf(f32),
    Node(&'a dyn Tomato),
}

#[cfg(feature = "alloc")]
pub trait Tomato: core::fmt::Debug {
    fn inner<'a>(&'a self) -> TomatoItem<'a>;
    fn name(&self) -> String {
        let dbg_repr = format!("{:?}", self);
        dbg_repr
            .split_once('(')
            .map(|(name, _)| name)
            .unwrap_or("-")
            .to_string()
    }
    fn id(&self) -> TomatoId;
}

#[cfg(feature = "alloc")]
macro_rules! all_nodes {
    ($name:ident; $variant:ident; $($var:ident),*) => {
        impl Tomato for $name {
            fn inner<'a>(&'a self) -> crate::TomatoItem<'a> {
                match self {
                    $(
                    $name::$var(inner) => crate::TomatoItem::Node(inner as &dyn Tomato)
                    ),*
                }
            }

            fn id(&self) -> crate::TomatoId {
                $variant::from(self) as crate::TomatoId
            }
        }
    };
}
#[cfg(feature = "alloc")]
pub(crate) use all_nodes;
#[cfg(feature = "alloc")]
all_nodes! {Reading; ReadingDiscriminants; LargeBedroom} //, Test}

#[derive(
    strum::EnumDiscriminants,
    strum::VariantNames,
    Clone,
    Debug,
    defmt::Format,
    Serialize,
    Deserialize,
    MaxSize,
)]
#[strum_discriminants(derive(Hash))]
pub enum Reading {
    LargeBedroom(large_bedroom::Reading),
    // SmallBedroom(small_bedroom::Reading),
    // Test,
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
    pub values: heapless::Vec<Reading, MAX_ITEMS>,
    pub version: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub error: Error,
    pub version: u8,
}

pub const SENSOR_MSG: u8 = 0;
pub const ERROR_REPORT: u8 = 1;

pub fn decode<const M: usize>(
    mut bytes: impl AsMut<[u8]>,
) -> Result<Result<SensorMessage<M>, ErrorReport>, DecodeError> {
    let bytes = bytes.as_mut();
    let msg_type = bytes[0];
    let bytes = &mut bytes[1..];

    if msg_type == SENSOR_MSG {
        Ok(Ok(SensorMessage::<M>::decode(&mut bytes[1..])?))
    } else if msg_type == ERROR_REPORT {
        Ok(Err(ErrorReport::decode(&mut bytes[1..])?))
    } else {
        Err(DecodeError::IncorrectMsgType(msg_type))
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "thiserror", derive(thiserror::Error))]
pub enum DecodeError {
    #[cfg_attr(feature = "thiserror", error("Could not decode SensorMessage: {0}"))]
    CorruptEncoding(postcard::Error),
    #[cfg_attr(
        feature = "thiserror",
        error("Got an unknown message type, expected zero or one got: {}")
    )]
    IncorrectMsgType(u8),
}

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
        postcard::from_bytes_cobs(bytes.as_mut()).map_err(DecodeError::CorruptEncoding)
    }

    pub fn version(&self) -> u8 {
        self.version
    }
}

impl ErrorReport {
    /// the 2x is the max overhead from COBS encoding the encoded data
    /// +2 is for the version
    /// +4 covers the length of the heapless list
    pub const ENCODED_SIZE: usize = 2 * (Error::POSTCARD_MAX_SIZE + 2);

    pub fn new(error: Error) -> Self {
        Self { error, version: 0 }
    }

    /// Buffer should be at least Self::ENCODED_SIZE long. The returned slice contains
    /// the serialized data. It can be shorter then the input buffer.
    pub fn encode_slice<'a>(&self, buf: &'a mut [u8]) -> &'a mut [u8] {
        postcard::to_slice_cobs(self, buf).expect("Encoding should not fail")
    }

    pub fn decode(mut bytes: impl AsMut<[u8]>) -> Result<Self, DecodeError> {
        postcard::from_bytes_cobs(bytes.as_mut()).map_err(DecodeError::CorruptEncoding)
    }
}

pub type ErrorString = heapless::String<200>;
// thiserror does not work on nostd. That will change when this lands:
// https://github.com/rust-lang/rust/issues/103765
// at that point try switching this to fmt::Display
pub fn make_error_string(e: impl core::fmt::Debug) -> ErrorString {
    defmt::info!("making error string");
    use core::fmt::Write;

    let mut s = ErrorString::new();
    core::write!(s, "{e:?}").ok();
    defmt::info!("done");
    s
}
