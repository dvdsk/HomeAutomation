use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::Error;

use super::{cobs_overhead, DecodeMsgError};

#[allow(clippy::large_enum_variant)] // can not use Box on embedded
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    pub error: Error,
    pub version: u8,
}

impl ErrorReport {
    /// +2 is for the version
    /// cobs encoding still needed
    const HALF_ENCODED_SIZE: usize = Error::POSTCARD_MAX_SIZE + 2;

    /// cobs and postcard encoded
    pub const ENCODED_SIZE: usize =
        Self::HALF_ENCODED_SIZE + cobs_overhead(Self::HALF_ENCODED_SIZE);

    #[must_use]
    pub fn new(error: Error) -> Self {
        Self { error, version: 0 }
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
}

pub type ErrorString = heapless::String<200>;
// thiserror does not work on nostd. That will change when this lands:
// https://github.com/rust-lang/rust/issues/103765
// at that point try switching this to fmt::Display
pub fn make_error_string(e: impl core::fmt::Debug) -> ErrorString {
    use core::fmt::Write;

    let mut s = ErrorString::new();
    core::write!(s, "{e:?}").ok();
    s
}
