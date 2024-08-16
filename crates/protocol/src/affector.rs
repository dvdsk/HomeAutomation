use postcard::accumulator::{CobsAccumulator, FeedResult};
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

use crate::large_bedroom;
use crate::msg::cobs_overhead;

#[derive(Clone, Copy, Debug, defmt::Format, Serialize, Deserialize, MaxSize)]
pub enum Affector {
    LargeBedroom(large_bedroom::Affector),
}

impl Affector {
    const ENCODED_SIZE: usize =
        Affector::POSTCARD_MAX_SIZE + cobs_overhead(Affector::POSTCARD_MAX_SIZE);
}

pub struct Decoder {
    cobs_buf: CobsAccumulator<{ Affector::ENCODED_SIZE }>,
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            cobs_buf: CobsAccumulator::new(),
        }
    }

    /// if this returns Some(_, remaining) move the remaining into it again
    pub fn feed<'a>(&mut self, read_bytes: &'a [u8]) -> Option<(Affector, &'a [u8])> {
        let mut window = read_bytes;
        while !window.is_empty() {
            window = match self.cobs_buf.feed::<Affector>(read_bytes) {
                FeedResult::Consumed => break,
                FeedResult::OverFull(new_window) => new_window,
                FeedResult::DeserError(new_window) => new_window,
                FeedResult::Success { data, remaining } => return Some((data, remaining)),
            }
        }
        None
    }
}
