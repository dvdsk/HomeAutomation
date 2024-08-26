#[cfg(feature = "alloc")]
use core::ops::Range;

use postcard::accumulator::{CobsAccumulator, FeedResult};
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::large_bedroom;
use crate::msg::cobs_overhead;

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
}

#[cfg(feature = "alloc")]
tree::all_nodes! {Affector; AffectorDiscriminants; LargeBedroom}

#[cfg(feature = "alloc")]
#[derive(Debug, Clone)]
pub struct Info {
    pub description: &'static str,
    /// affectors not related to a reading
    /// and therefore not related to a read device
    pub free_affectors: &'static [Affector],
}

#[cfg(feature = "alloc")]
pub enum ControlValue<'a> {
    Trigger,
    /// if you need floats just map this to a float
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
    const ENCODED_SIZE: usize =
        Affector::POSTCARD_MAX_SIZE + cobs_overhead(Affector::POSTCARD_MAX_SIZE);

    #[cfg(feature = "alloc")]
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        postcard::to_allocvec_cobs(self).expect("Encoding should not fail")
    }

    // info() is explicitly not defined, use the tree impl to get at it
    #[cfg(feature = "alloc")]
    pub fn controls(&mut self) -> Vec<Control> {
        match self {
            Affector::LargeBedroom(a) => a.controls(),
        }
    }
}

pub struct Decoder {
    cobs_buf: CobsAccumulator<{ Affector::ENCODED_SIZE }>,
}

impl Default for Decoder {
    fn default() -> Self {
        Self {
            cobs_buf: CobsAccumulator::new(),
        }
    }
}

impl Decoder {
    /// if this returns Some(_, remaining) move the remaining into it again
    pub fn feed<'a>(&mut self, read_bytes: &'a [u8]) -> Option<(Affector, &'a [u8])> {
        let mut window = read_bytes;
        while !window.is_empty() {
            window = match self.cobs_buf.feed::<Affector>(read_bytes) {
                FeedResult::Consumed => break,
                FeedResult::OverFull(new_window) | FeedResult::DeserError(new_window) => new_window,
                FeedResult::Success { data, remaining } => return Some((data, remaining)),
            }
        }
        None
    }
}
