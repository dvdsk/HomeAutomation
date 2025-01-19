#[inline]
#[allow(clippy::cast_possible_truncation)]
pub fn decode(line: &[u8], bit_offset: u8, length: u8) -> u32 {
    let start_byte = (bit_offset / 8) as usize;
    let stop_byte = ((bit_offset + length) / 8) as usize;

    let start_mask: u8 = !0 >> (bit_offset % 8);
    let used_bits = bit_offset + length - stop_byte as u8 * 8;
    let stop_mask = !(!0 >> used_bits);

    //decode first bit (never needs shifting (lowest part is used))
    let mut decoded: u32 = u32::from(line[start_byte] & start_mask);
    let mut bits_read = 8 - (bit_offset % 8);
    //if we have more bits
    if length > 8 {
        //decode middle bits, no masking needed
        for byte in &line[start_byte + 1..stop_byte] {
            decoded |= u32::from(*byte) << bits_read;
            bits_read += 8;
        }
    }
    let stop_byte = ((bit_offset + length).div_ceil(8) - 1) as usize; //starts at 0
    decoded |= u32::from(line[stop_byte] & stop_mask) << (bits_read - (8 - used_bits));

    decoded
}

#[inline]
#[allow(clippy::cast_possible_truncation)]
/// Encode the lower `length` bits of `to_encode` in a zero-ed byte slice
/// starting at the `bit_offset` bit.
///
/// If `bit_offset` mod 8 is not zero this means we skip the that amount of
/// lower bits in the first byte used for encoding. Inversely
/// if `bit_offset` + `length` mod 8 is not zero that means we only use
/// that amount of lower bits of the last byte.
///
/// # The first byte
/// Only using the free bits in the first byte is done by first bitmasking on
/// the lower 8 bits of `to_encode`. That mask when AND-ed makes the highest
/// bits zero. Specifically the `bit_offset` mod 8 highest bits are made zero.
///
/// Then the remaining bits are shifted up such that they fill out the free
/// space in the highest bits of the first byte
///
/// Example: store 8 bits starting at bit 4:
///     bit_offset: 4, used: 4 mod 8 = 4
///     length: 8
///     bits_in_first: 8 - (4 mod 8) = 4
///     mask: 0b0000_1111
///     shift: 4 up
///
/// Example mask, store 4 bits starting at bit 5:
///     bit_offset: 5, used: 5 mod 8 = 5
///     length: 4
///     bits_in_first: 8 - (5 mod 8) = 3
///     mask: 0b0000_0111 # Note how only 3 bits are selected
///     shift: 5 up
///
/// # The last byte
/// We fit `bit_offest` mod 8 bits in the first byte, lets call that `bits_in_first`.
/// Then N * 8 more in the bytes in between the first and last byte used. This leaves
/// (`length` - `bits_in_first`) mod 8 bits to still encode. These are the highest
/// bits in `to_encode`. We again select them using a mask and AND.
///
/// This mask is ones for the `bits_in_last` highest bits from `to_encode` and
/// zeros otherwise. Then the bits are shifted all the way down so they fit
/// the lower bits of the last byte in the slice used to store `to_encode`.
///
/// Example: store 20 bits starting at bit 4
///     bit_offset: 4
///     length: 22
///     bits_in_first: 4
///     bits_in_last: (length - bits_in_first) mod 8 = 2
///     mask: 0b0000_0000__0011_0000__0000_0000__0000_0000
///     shift: 20 down
///
/// Example: store 7 bits starting at bit 13
///     bit_offset: 13
///     length: 7
///     bits_in_first: 8 - (5 mod 8) = 3
///     bits_in_last: (length - bits_in_first) mod 8 = 4
///     mask: 0b0000_0000__0000_0000__0000_0000__0111_1000
///     shift: 7 down
///
pub fn encode(to_encode: u32, line: &mut [u8], bit_offset: u8, length: u8) {
    let start_mask = !0 >> (bit_offset % 8);

    let start_byte = (bit_offset / 8) as usize;
    let stop_byte = ((bit_offset + length) / 8) as usize;

    //encode first bit (never needs shifting (lowest part is used))
    line[start_byte] |= (to_encode as u8) & start_mask;
    let mut bits_written = 8 - (bit_offset % 8);

    if length > 8 {
        //decode middle bits, no masking needed
        for byte in &mut line[start_byte + 1..stop_byte] {
            *byte |= (to_encode >> bits_written) as u8;
            bits_written += 8;
        }
    }

    let used_bits = bit_offset + length - stop_byte as u8 * 8;
    let stop_mask = !(!0 >> used_bits);
    let stop_byte = (bit_offset + length).div_ceil(8) as usize; //starts at 0
    line[stop_byte - 1] |= (to_encode >> (bits_written - (8 - used_bits))) as u8 & stop_mask;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_and_decode_multiple_edge_case() {
        let mut line = vec![0, 0, 0, 0, 0, 0, 0, 0];
        encode(1, &mut line, 0, 8);

        print!("binary repr: ");
        for byte in &line {
            print!("{:b}, ", byte);
        }
        println!();

        encode(2, &mut line, 8, 8);

        print!("binary repr: ");
        for byte in &line {
            print!("{:b}, ", byte);
        }
        println!();

        let decoded1 = decode(&line, 0, 8);
        let decoded2 = decode(&line, 8, 8);

        println!("0-10 {} {:b}", decoded1, decoded1);
        println!("10-20 {} {:b}", decoded2, decoded2);
        assert_eq!(decoded1, 1);
        assert_eq!(decoded2, 2);
    }

    #[test]
    fn encode_and_decode_edge_case() {
        let test_case = u32::max_value();
        let mut line = vec![0, 0, 0, 0];
        encode(test_case, &mut line, 0, 32);

        print!("binary repr: ");
        for byte in &line {
            print!("{:b}, ", byte);
        }
        println!();

        let decoded1 = decode(&line, 0, 32);

        println!("0-{} {} {:b}", 32, decoded1, decoded1);
        assert_eq!(decoded1, test_case);
    }

    #[test]
    fn encode_and_decode_start_case() {
        let test_case = 1023;
        let mut line = vec![0, 0, 0, 0];
        encode(test_case, &mut line, 0, 10);

        print!("binary repr: ");
        for byte in &line {
            print!("{:b}, ", byte);
        }
        println!();

        print!("array repr: ");
        for byte in &line {
            print!("{}, ", byte);
        }
        println!();

        let decoded1 = decode(&line, 0, 10);

        println!("0-{} {} {:b}", 10, decoded1, decoded1);
        assert_eq!(decoded1, test_case);
    }

    #[test]
    #[ignore = "takes 10 secs to run"]
    fn encode_and_decode_multiple() {
        for length in 8..33 {
            for offset in 0..16 {
                for _power1 in 0..length as u16 * 10 {
                    for _power2 in 0..length as u16 * 10 {
                        let power1 = _power1 as f32 * 0.1;
                        let power2 = _power2 as f32 * 0.1;
                        let mut array = vec![0; 12];
                        let test_numb1 = 2f32.powf(power1) as u32;
                        let test_numb2 = 2f32.powf(power2) as u32;
                        encode(test_numb1, array.as_mut_slice(), offset, length);
                        encode(test_numb2, array.as_mut_slice(), offset + length, length);

                        let decoded_test_numb1 = decode(array.as_slice(), offset, length);
                        let decoded_test_numb2 = decode(array.as_slice(), offset + length, length);
                        assert_eq!(
                            test_numb1, decoded_test_numb1,
                            "\n##### numb:1, \noffset: {},\nlength: {}, \nvalue1: {}, \nvalue2: {}",
                            offset, length, test_numb1, test_numb2
                        );
                        assert_eq!(
                            test_numb2,
                            decoded_test_numb2,
                            "\n##### numb:2, \noffset: {},\nlength: {}, \nvalue1: {}, \nvalue2: {}",
                            offset + length,
                            length,
                            test_numb1,
                            test_numb2
                        );
                    }
                }
            }
        }
    }

    fn print_vec_bin(array: Vec<u8>) -> String {
        let mut outstr = String::from("binary repr: ");
        for byte in &array {
            outstr.push_str(&format!("{:b}, ", byte));
        }
        outstr.push_str("\n");
        outstr
    }

    #[test]
    fn encode_and_decode() {
        for length in 8..32 {
            for offset in 0..16 {
                for _power in 0..length as u16 * 10 {
                    let power = _power as f32 * 0.1;
                    let mut array = vec![0; 8];
                    let test_numb = 2f32.powf(power) as u32;
                    encode(test_numb, array.as_mut_slice(), offset, length);

                    let decoded_test_numb = decode(array.as_slice(), offset, length);
                    assert_eq!(
                        test_numb,
                        decoded_test_numb,
                        "offset: {}, length: {} {}",
                        offset,
                        length,
                        print_vec_bin(array)
                    );
                }
            }
        }
    }
}
