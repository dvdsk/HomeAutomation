use crate::data::series::bitspec;

#[derive(Debug, Clone)]
pub(crate) struct ExportDecoder {
    pub(crate) fields: Vec<bitspec::Field<f32>>,
}

impl ExportDecoder {
    pub(crate) fn from_fields(fields: Vec<bitspec::Field<f32>>) -> Self {
        Self { fields }
    }
}

impl byteseries::Decoder for ExportDecoder {
    type Item = smallvec::SmallVec<f32, 8>;

    fn decode_payload(&mut self, payload: &[u8]) -> Self::Item {
        self.fields
            .iter()
            .map(|field| field.decode(payload))
            .collect()
    }
}
