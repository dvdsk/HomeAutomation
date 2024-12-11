#[cfg(feature = "alloc")]
use core::ops::Range;

use postcard::accumulator::{CobsAccumulator, FeedResult};
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::msg::cobs_overhead;
use crate::{large_bedroom, small_bedroom, DecodeMsgError};

#[cfg(feature = "alloc")]
pub mod tree;

#[derive(
    strum::EnumDiscriminants,
    Clone,
    Copy,
    Debug,
    defmt::Format,
    Serialize,
    Deserialize,
    MaxSize,
    PartialEq,
    Eq,
    Hash,
)]
#[strum_discriminants(derive(Hash))]
pub enum Affector {
    LargeBedroom(large_bedroom::Affector),
    SmallBedroom(small_bedroom::Affector),
}

#[cfg(feature = "alloc")]
tree::all_nodes! {Affector; AffectorDiscriminants; LargeBedroom, SmallBedroom}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub struct Info {
    pub description: &'static str,
}

#[cfg(feature = "alloc")]
pub enum ControlValue<'a> {
    Trigger,
    /// If you need floats just map this to a float
    /// in the affector device like: `range = range/x + y`
    SetNum {
        valid_range: Range<u64>,
        /// FnOnce such that value does become outdated
        setter: Option<Box<dyn FnOnce(usize) + 'a>>,
        value: usize,
    },
}

#[cfg(feature = "alloc")]
pub struct Control<'a> {
    pub name: &'static str,
    pub value: ControlValue<'a>,
}

impl Affector {
    pub const ENCODED_SIZE: usize =
        Affector::POSTCARD_MAX_SIZE + cobs_overhead(Affector::POSTCARD_MAX_SIZE);

    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec_cobs(self).expect("Encoding should not fail")
    }

    pub fn decode(mut bytes: impl AsMut<[u8]>) -> Result<Self, DecodeMsgError> {
        postcard::from_bytes_cobs(bytes.as_mut()).map_err(DecodeMsgError::CorruptEncoding)
    }

    // Info() is explicitly not defined, use the tree impl to get at it
    #[cfg(feature = "alloc")]
    pub fn controls(&mut self) -> Vec<Control> {
        match self {
            Affector::LargeBedroom(a) => a.controls(),
            Affector::SmallBedroom(a) => a.controls(),
        }
    }
}

pub struct Decoder {
    cobs_buf: CobsAccumulator<{ 2 * Affector::ENCODED_SIZE }>,
}

impl Default for Decoder {
    fn default() -> Self {
        Self {
            cobs_buf: CobsAccumulator::new(),
        }
    }
}

#[derive(Debug, defmt::Format)]
pub struct DeserializeError;

impl Decoder {
    /// If this returns Some(_, remaining) move the remaining into it again
    pub fn feed<'a>(
        &mut self,
        read_bytes: &'a [u8],
    ) -> Result<Option<(Affector, &'a [u8])>, DeserializeError> {
        let mut window = read_bytes;
        while !window.is_empty() {
            window = match self.cobs_buf.feed::<Affector>(read_bytes) {
                FeedResult::Consumed => break,
                FeedResult::OverFull(new_window) => new_window,
                FeedResult::DeserError(_) => return Err(DeserializeError),
                FeedResult::Success { data, remaining } => return Ok(Some((data, remaining))),
            }
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessage<const MAX_ITEMS: usize> {
    pub values: heapless::Vec<Affector, MAX_ITEMS>,
    pub version: u8,
}

impl<const MAX_ITEMS: usize> ListMessage<MAX_ITEMS> {
    /// +2 is for the version
    /// +4 covers the length of the heapless list
    pub const HALF_ENCODED_SIZE: usize = (MAX_ITEMS * Affector::POSTCARD_MAX_SIZE + 2 + 4);
    /// cobs and postcard encoded
    pub const ENCODED_SIZE: usize =
        Self::HALF_ENCODED_SIZE + cobs_overhead(Self::HALF_ENCODED_SIZE);

    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec_cobs(self).expect("Encoding should not fail")
    }

    /// Buffer should be at least `Self::ENCODED_SIZE` long. The returned slice contains
    /// the serialized data. It can be shorter then the input buffer.
    #[must_use]
    pub fn encode_slice<'a>(&self, buf: &'a mut [u8]) -> &'a mut [u8] {
        postcard::to_slice_cobs(self, buf).expect("Encoding should not fail")
    }

    #[must_use]
    pub fn empty() -> Self {
        Self {
            values: heapless::Vec::new(),
            version: 1,
        }
    }

    pub fn decode(mut bytes: impl AsMut<[u8]>) -> Result<Self, DecodeMsgError> {
        postcard::from_bytes_cobs(bytes.as_mut()).map_err(DecodeMsgError::CorruptEncoding)
    }
}

#[cfg(all(test, feature = "alloc"))]
mod test {
    use super::*;
    use large_bedroom::bed;

    #[test]
    fn decoder_decodes_encoded() {
        let test_affector =
            Affector::LargeBedroom(large_bedroom::Affector::Bed(bed::Affector::RgbLed {
                red: 0,
                green: 5,
                blue: 0,
            }));

        let encoded = test_affector.encode();
        let mut encoded_copy = encoded.clone();
        let decoded: Affector = postcard::from_bytes_cobs(encoded_copy.as_mut_slice()).unwrap();
        assert_eq!(decoded, test_affector);

        let mut decoder = Decoder::default();

        decoder.feed(&encoded).unwrap();
        let res = decoder.feed(&encoded).unwrap();

        let empty_slice = [].as_slice();
        assert_eq!(res, Some((test_affector, empty_slice)));
    }
}
