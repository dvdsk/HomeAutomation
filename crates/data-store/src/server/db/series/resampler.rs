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

#[cfg(test)]
mod test {
    use byteseries::{Decoder, Encoder};
    use smallvec::SmallVec;

    use super::*;

    #[test]
    fn decoded_is_correct() {
        let reading = protocol::Reading::LargeBedroom(protocol::large_bedroom::Reading::Bed(
            protocol::large_bedroom::bed::Reading::Temperature(0.5),
        ));

        let readings = reading.device().affected_readings();
        // dbg!(readings.iter().map(Tree::leaf).collect::<Vec<_>>());
        let specs = crate::server::db::series::to_speclist(readings);
        // dbg!(&specs);
        let fields = bitspec::speclist_to_fields(specs);

        let payload_size = fields
            .iter()
            .map(|spec| spec.length as usize)
            .sum::<usize>()
            .div_ceil(8);
        let mut resampler = Resampler::from_fields(fields.clone(), payload_size);
        let mut item: SmallVec<f32, 8> = SmallVec::new();
        item.push(0.5f32);
        item.push(0.5f32);
        let bytes = resampler.encode_item(&item);
        let decoded = resampler.decode_payload(&bytes);
        assert!(
            decoded
                .iter()
                .zip(item.clone())
                .all(|(a, b)| (a - b).abs() < 0.01),
            "decoded and original different, decoded: {decoded:?}, org: {item:?}"
        );
    }
}
