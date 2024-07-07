use std::ops::Range;

use serde::{Deserialize, Serialize};

use super::MetaField;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct RangeWithRes {
    pub range: Range<f32>,
    pub resolution: f32,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct LengthWithOps {
    pub(crate) length: u8,
    pub(crate) decode_scale: f32,
    pub(crate) decode_add: f32,
}

impl From<RangeWithRes> for LengthWithOps {
    fn from(field: RangeWithRes) -> Self {
        let given_range = field.range.end - field.range.start;
        let needed_range = given_range as f32 / field.resolution as f32;
        let length = needed_range.log2().ceil() as u8;
        let decode_scale = field.resolution;

        let decode_add = field.range.start;
        LengthWithOps {
            decode_scale,
            length,
            decode_add,
        }
    }
}

pub fn speclist_to_fields(input: Vec<LengthWithOps>) -> Vec<MetaField<f32>> {
    let mut res = Vec::new();

    let mut start_bit = 0;
    for field in input {
        res.push(MetaField::<f32> {
            offset: start_bit,
            length: field.length,
            decode_scale: field.decode_scale,
            decode_add: field.decode_add,
        });
        start_bit = start_bit
            .checked_add(field.length)
            .expect("line longer the 256 bits are not supported");
    }

    res
}
