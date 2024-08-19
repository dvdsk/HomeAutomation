#![allow(clippy::module_name_repetitions)]

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::Reading;

use super::{cobs_overhead, DecodeMsgError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorMessage<const MAX_ITEMS: usize> {
    pub values: heapless::Vec<Reading, MAX_ITEMS>,
    pub version: u8,
}

impl<const MAX_ITEMS: usize> Default for SensorMessage<MAX_ITEMS> {
    fn default() -> Self {
        Self {
            values: heapless::Vec::new(),
            version: 0,
        }
    }
}
impl<const MAX_ITEMS: usize> SensorMessage<MAX_ITEMS> {
    /// +2 is for the version
    /// +4 covers the length of the heapless list
    pub const HALF_ENCODED_SIZE: usize = (MAX_ITEMS * Reading::POSTCARD_MAX_SIZE + 2 + 4);

    /// cobs and postcard encoded
    pub const ENCODED_SIZE: usize =
        Self::HALF_ENCODED_SIZE + cobs_overhead(Self::HALF_ENCODED_SIZE);

    #[must_use]
    pub fn space_left(&self) -> bool {
        self.values.len() < self.values.capacity()
    }

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

    pub fn decode(mut bytes: impl AsMut<[u8]>) -> Result<Self, DecodeMsgError> {
        postcard::from_bytes_cobs(bytes.as_mut()).map_err(DecodeMsgError::CorruptEncoding)
    }

    #[must_use]
    pub fn version(&self) -> u8 {
        self.version
    }
}
