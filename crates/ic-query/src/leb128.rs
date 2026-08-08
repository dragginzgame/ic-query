//! Module: leb128
//!
//! Responsibility: decode canonical unsigned `u64` LEB128 values.
//! Does not own: certificate paths, authority-specific errors, or report fields.
//! Boundary: rejects truncation, overflow, trailing bytes, and non-minimal encodings.

pub fn decode_canonical_unsigned_u64(field: &str, bytes: &[u8]) -> Result<u64, String> {
    let mut value = 0_u64;
    let mut shift = 0_u32;
    for (index, byte) in bytes.iter().copied().enumerate() {
        let low = u64::from(byte & 0x7f);
        let shifted = low
            .checked_shl(shift)
            .ok_or_else(|| format!("{field} unsigned LEB128 value overflows u64"))?;
        value = value
            .checked_add(shifted)
            .ok_or_else(|| format!("{field} unsigned LEB128 value overflows u64"))?;
        if byte & 0x80 == 0 {
            if index + 1 != bytes.len() || encode_unsigned_u64(value) != bytes {
                return Err(format!("{field} is not canonical unsigned LEB128"));
            }
            return Ok(value);
        }
        shift = shift
            .checked_add(7)
            .ok_or_else(|| format!("{field} unsigned LEB128 value overflows u64"))?;
    }
    Err(format!("{field} is truncated unsigned LEB128"))
}

pub fn encode_unsigned_u64(mut value: u64) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(10);
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            return bytes;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_values_round_trip_at_u64_boundaries() {
        for value in [0, 1, 127, 128, u64::from(u32::MAX), u64::MAX] {
            let bytes = encode_unsigned_u64(value);
            assert_eq!(
                decode_canonical_unsigned_u64("value", &bytes).expect("canonical value"),
                value
            );
        }
    }

    #[test]
    fn malformed_encodings_are_rejected() {
        for bytes in [
            &[][..],
            &[0x80][..],
            &[0x80, 0x00][..],
            &[0x00, 0x00][..],
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x02][..],
        ] {
            assert!(decode_canonical_unsigned_u64("value", bytes).is_err());
        }
    }
}
