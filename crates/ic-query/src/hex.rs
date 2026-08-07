#[must_use]
pub fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[usize::from(byte >> 4)] as char);
        out.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    out
}

#[must_use]
pub fn is_lowercase_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

#[must_use]
pub fn is_canonical_lowercase_hex(value: &str) -> bool {
    !value.is_empty() && value.len().is_multiple_of(2) && is_lowercase_hex(value)
}

///
/// decode_lowercase_hex
///
/// Decode even-length lowercase hexadecimal, including an empty byte string.
///

#[cfg(feature = "certified-subnet-catalog-host")]
pub fn decode_lowercase_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) || !is_lowercase_hex(value) {
        return None;
    }
    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowercase_hex_predicates_distinguish_alphabet_and_canonical_length() {
        for (value, lowercase, canonical) in [
            ("", true, false),
            ("0", true, false),
            ("00", true, true),
            ("0a", true, true),
            ("0A", false, false),
            ("0g", false, false),
            ("é", false, false),
        ] {
            assert_eq!(is_lowercase_hex(value), lowercase, "{value:?}");
            assert_eq!(is_canonical_lowercase_hex(value), canonical, "{value:?}");
        }
    }

    #[cfg(feature = "nns-host")]
    #[test]
    fn lowercase_hex_decoder_accepts_complete_bytes_only() {
        assert_eq!(decode_lowercase_hex(""), Some(Vec::new()));
        assert_eq!(decode_lowercase_hex("000aff"), Some(vec![0, 10, 255]));
        assert_eq!(decode_lowercase_hex("0"), None);
        assert_eq!(decode_lowercase_hex("0A"), None);
        assert_eq!(decode_lowercase_hex("0g"), None);
    }
}
