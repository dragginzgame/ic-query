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
#[cfg(feature = "host")]
pub fn is_lowercase_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

#[must_use]
#[cfg(feature = "host")]
pub fn is_canonical_lowercase_hex(value: &str) -> bool {
    !value.is_empty() && value.len().is_multiple_of(2) && is_lowercase_hex(value)
}

#[cfg(all(test, feature = "host"))]
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
}
