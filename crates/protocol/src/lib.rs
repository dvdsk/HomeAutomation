#![cfg_attr(not(feature = "alloc"), no_std)]
pub mod large_bedroom;

use postcard::experimental::max_size::MaxSize as _;
use serde::{Deserialize, Serialize};

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
    ($name:ident {$($variant:ident,)*}) => {
        #[derive(
            Clone,
			Copy,
			Debug,
			serde::Serialize,
			serde::Deserialize,
			PartialEq,
			Eq,
            postcard::experimental::max_size::MaxSize
        )]
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
    PartialEq,
    Eq,
    postcard::experimental::max_size::MaxSize,
)]
pub struct Press(u16);

impl Press {
    pub fn is_long(&self) -> bool {
        if self.0 > 300 {
            true
        } else {
            false
        }
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    postcard::experimental::max_size::MaxSize,
)]
pub enum SensorValue {
    LargeBedroom(large_bedroom::LargeBedroom),
    #[cfg(test)]
    Test,
}

impl SensorValue {
    pub fn version() -> u8 {
        0u8
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorMessage<const MAX_ITEMS: usize> {
    pub values: heapless::Vec<SensorValue, MAX_ITEMS>,
    pub version: u8,
}

impl<const MAX_ITEMS: usize> SensorMessage<MAX_ITEMS> {
    /// the 2x is the max overhead from COBS encoding the encoded data
    /// +2 is for the version
    /// +4 covers the length of the heapless list
    pub const ENCODED_SIZE: usize = 2 * (MAX_ITEMS * SensorValue::POSTCARD_MAX_SIZE + 2 + 4);

    #[cfg(feature = "alloc")]
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec_cobs(self).expect("Encoding should not fail")
    }

    /// Buffer should be at least Self::ENCODED_SIZE long. The returned slice contains
    /// the serialized data. It can be shorter then the input buffer.
    pub fn encode_slice<'a, const N: usize>(&self, buf: &'a mut [u8]) -> &'a mut [u8] {
        postcard::to_slice_cobs(self, buf).expect("Encoding should not fail")
    }

    pub fn decode(mut bytes: impl AsMut<[u8]>) -> Result<Self, postcard::Error> {
        postcard::from_bytes_cobs(bytes.as_mut())
    }

    pub fn version() -> u8 {
        0u8
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decode_from_different_max_items() {
        let mut values = heapless::Vec::new();
        values.push(SensorValue::Test).unwrap();
        let mut bytes = SensorMessage::<1> {
            values: values.clone(),
            version: 0u8,
        }
        .encode();

        let decoded: SensorMessage<100> = SensorMessage::<100>::decode(&mut bytes).unwrap();

        assert_eq!(decoded.values.to_vec(), values.to_vec());
    }
}
