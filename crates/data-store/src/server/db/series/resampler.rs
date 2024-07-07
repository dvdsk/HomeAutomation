use super::bitspec;
use smallvec::smallvec;

#[derive(Debug, Clone)]
pub(crate) struct Resampler {
    pub(crate) fields: Vec<bitspec::MetaField<f32>>,
    pub(crate) payload_size: usize,
}

impl Resampler {
    pub(crate) fn from_fields(fields: Vec<bitspec::MetaField<f32>>, payload_size: usize) -> Self {
        Self {
            payload_size,
            fields,
        }
    }
}

impl byteseries::Resampler for Resampler {
    type State = smallvec::SmallVec<f32, 8>;

    fn state(&self) -> Self::State {
        smallvec![0f32; self.fields.len()]
    }
}

impl byteseries::Decoder for Resampler {
    type Item = smallvec::SmallVec<f32, 8>;

    fn decode_payload(&mut self, payload: &[u8]) -> Self::Item {
        self.fields
            .iter()
            .map(|field| field.decode(payload))
            .collect()
    }
}

impl byteseries::Encoder for Resampler {
    type Item = smallvec::SmallVec<f32, 8>;

    // PERF: should probably take a &mut[u8] instead of returning vec
    // <dvdsk noreply@davidsk.dev>
    fn encode_item(&mut self, item: &Self::Item) -> Vec<u8> {
        let mut encoded = vec![0u8; self.payload_size];
        for (field, item) in self.fields.iter().zip(item) {
            field.encode::<f32>(*item, &mut encoded);
        }
        encoded
    }
}
