/// ```txt
/// array's with bytes in them follow Little Endian byte order
/// (not relevant) to the algo but might be nice to know. It can
/// get a little confusing given the lowest bit is printed right most
/// given these strange bit 1 is next to bit 16 situations. See diagram:
///
///      byte 1              byte 2                  byte 3
/// [ 8|7|6|5|4|3|2|1, 16|15|14|13|12|11|10|9, 24|23|22|21|20|19|18|17 ]
///          |____________________________________________|
/// |________|                        encoded B
///  encoded   lower part of B                    higher part of B
///   value    stored in lower part               stored in higher
///     A      of byte 1                          part of byte 3
///
/// B: length = 16, offest = 4, at byte 3 bits read = 12
/// ```

#[inline]
#[allow(clippy::cast_possible_truncation)]
/// Decodes `length` bytes starting at `bit_offset` from `line`.
pub fn decode(line: &[u8], bit_offset: u8, length: u8) -> u32 {
    let first_byte = (bit_offset / 8) as usize;

    // build mask to get lowest bits of first byte
    let start_mask: u8 = !0 >> (bit_offset % 8);
    // second mask for sources with length smaller then 8 bits
    let start_mask = if length < (bit_offset % 8) {
        start_mask & !(!0 << length)
    } else {
        start_mask
    };

    //decode first bit (never needs shifting (lowest part is used))
    let mut decoded: u32 = u32::from(line[first_byte] & start_mask);
    let mut bits_read = (8 - (bit_offset % 8)).min(length);

    let last_byte_new = ((bit_offset + length).div_ceil(8)) as usize;
    let last_byte_idx = last_byte_new - 1;
    if length > 8 {
        //decode middle bits, no masking needed
        for byte in &line[first_byte + 1..last_byte_idx] {
            decoded |= u32::from(*byte) << bits_read;
            bits_read += 8;
        }
    }

    // build mask to get highest bits of last byte
    let bits_still_needed = length - bits_read;
    // shifting 8 bits to get mask 0b1111_1111 panics with `u8`
    let end_mask = if bits_still_needed == 8 {
        0b1111_1111
    } else {
        !(!0 >> bits_still_needed)
    };

    // let bits_in_last_byte = bits_still_needed;
    let shift_back = 8 - bits_still_needed;
    let shift_in_position = bits_read;

    decoded |= u32::from(line[last_byte_idx] & end_mask) >> shift_back
        << shift_in_position;

    decoded
}

#[inline]
#[allow(clippy::cast_possible_truncation)]
/// Encode the lower `length` bits of `to_encode` in a zero-ed byte slice
/// starting at the `bit_offset` bit.
///
/// If `bit_offset` mod 8 is not zero this means we skip the that amount of
/// higher bits in the first byte used for encoding. Inversely
/// if `bit_offset` + `length` mod 8 is not zero that means we only use
/// that amount of lower bits of the last byte.
///
/// # The first byte
/// Only using the free lower bits in the first byte is done by masking off
/// the higher part of the lower 8 bits of `to_encode`. That mask when AND-ed
/// makes the highest bits zero. Specifically the `bit_offset` mod 8 highest
/// bits are made zero.
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

    let first_byte = (bit_offset / 8) as usize;

    //encode first bit (never needs shifting (lowest part is used))
    line[first_byte] |= (to_encode as u8) & start_mask;
    let mut bits_written = (8 - (bit_offset % 8)).min(length);

    // this writes the last bit too when the offset + length is a multiple of 8
    // that is okay since then the byte is full used so no masking is needed.
    let last_byte = ((bit_offset + length) / 8) as usize;
    if length > 8 {
        //decode middle bits, no masking needed
        for byte in &mut line[first_byte + 1..last_byte] {
            *byte |= (to_encode >> bits_written) as u8;
            bits_written += 8;
        }
    }

    let used_bits = bit_offset + length - last_byte as u8 * 8;
    let end_mask = !(!0 >> used_bits);
    let last_byte = (bit_offset + length).div_ceil(8) as usize; //starts at 0

    if bits_written < length {
        // lets say to_encode is: 1100_0000_0000
        // and that 11 still needs to be encoded (bits written = 10, length = 12)
        // then we should shift them all the way down (shift 10) and then up into
        // the place where they are stored in the last byte. In this case the upper
        // 2 bits. So shift back up 6. Total shift is 4 down/right or:
        //
        // bits_written 10 - (8 - bits to use in last byte)
        //
        // now if we encode is: 11
        // and only the left most (higher) 1 still needs to be encoded. But it
        // will need to be encoded in the highest bit in the next byte. So it
        // must be shifted 7 up/left.
        let to_shift = bits_written as i16 - (8 - used_bits as i16);
        if to_shift > 0 {
            line[last_byte - 1] |= (to_encode >> to_shift) as u8 & end_mask;
        } else {
            line[last_byte - 1] |=
                (to_encode << to_shift.abs()) as u8 & end_mask;
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

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

    #[rstest]
    fn encode_and_decode_max(
        #[values(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17)]
        length: u8,
    ) {
        // > 2 bytes
        for offset in 0..=16 {
            let max_we_can_store = 2u32.pow(length as u32 - 1);

            let mut array = vec![0; 12];
            encode(max_we_can_store, array.as_mut_slice(), offset, length);
            let decoded = decode(array.as_slice(), offset, length);

            assert_eq!(max_we_can_store, decoded, "should decode {max_we_can_store}, got {decoded}, \noffset: {offset},\nlength: {length}"
                );
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
    fn encode_and_decode_various_numbers() {
        for length in 8..32 {
            for offset in 0..16 {
                for _power in 0..length as u16 * 10 {
                    let power = _power as f32 * 0.1;
                    let mut array = vec![0; 8];
                    let test_numb = 2f32.powf(power) as u32;
                    encode(test_numb, array.as_mut_slice(), offset, length);

                    let decoded_test_numb =
                        decode(array.as_slice(), offset, length);
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

    #[test]
    fn edge_case_1() {
        let line = [206, 84, 2, 0];
        let offset = 13;
        let length = 7;
        decode(&line, offset, length);
    }

    #[test]
    fn edge_case_2() {
        let line = [206, 84, 2, 0];
        let offset = 20;
        let length = 2;
        decode(&line, offset, length);
    }
}
