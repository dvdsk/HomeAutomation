use serde::{Deserialize, Serialize};

use super::compression;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Field<T> {
    pub offset: u8, //bits
    pub length: u8, //bits (max 32 bit variables)

    pub decode_scale: T,
    pub decode_add: T,
}

#[allow(dead_code)]
impl<T> Field<T>
where
    T: num::cast::NumCast
        + core::ops::Add
        + core::ops::SubAssign
        + core::ops::DivAssign
        + core::ops::MulAssign
        + core::marker::Copy
        + core::fmt::Display
        + core::fmt::Debug,
{
    pub fn decode<D>(&self, line: &[u8]) -> D
    where
        D: num::cast::NumCast
            + core::fmt::Display
            + core::ops::Add
            + core::ops::SubAssign
            + core::ops::MulAssign
            + core::ops::AddAssign,
    {
        let int_repr: u32 = compression::decode(line, self.offset, self.length);
        let mut decoded: D = num::cast(int_repr).unwrap();

        decoded *= num::cast(self.decode_scale).unwrap(); //FIXME flip decode scale / and *
        decoded += num::cast(self.decode_add).unwrap();

        decoded
    }
    pub fn encode(&self, mut numb: T, line: &mut [u8])
    where
        T: num::cast::NumCast
            + core::fmt::Display
            + core::ops::Add
            + core::ops::SubAssign
            + core::ops::AddAssign
            + core::ops::DivAssign,
    {
        numb -= num::cast(self.decode_add).unwrap();
        numb /= num::cast(self.decode_scale).unwrap();

        let to_encode: u32 = num::cast(numb).expect(&format!(
            "could not cast numb to u32, numb: {numb}, field: {self:?}",
        ));

        compression::encode(to_encode, line, self.offset, self.length);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let fields = &[
            // Ble_reliability_testing_dataset
            Field::<f32> {
                // Sine
                decode_add: -5000.0000000000,
                decode_scale: 1.0000000000,
                length: 14,
                offset: 0,
            },
            Field::<f32> {
                // Triangle
                decode_add: -10.0000000000,
                decode_scale: 0.0500000007,
                length: 10,
                offset: 14,
            },
        ];

        for i in 0..100 {
            let sine = -5000.0 + i as f32 * (5000.0 * 2.0) / 100.0;
            let triangle = 20.0 - i as f32 * (20.0 + 10.0) / 100.0;

            let mut line = [0u8, 0, 0];
            fields[0].encode(sine, &mut line);
            fields[1].encode(triangle, &mut line);

            let decoded_sine: f32 = fields[0].decode(&line);
            let decoded_triangle: f32 = fields[1].decode(&line);

            assert!(sine - decoded_sine <= 1. + 0.001);
            assert!(triangle - decoded_triangle <= 0.05 + 0.001);
        }
    }
}
