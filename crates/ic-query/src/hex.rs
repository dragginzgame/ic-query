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
pub fn is_lowercase_hex_32_bytes(value: &str) -> bool {
    value.len() == 64 && is_lowercase_hex(value)
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

#[cfg(any(test, feature = "sns-host", feature = "subnet-catalog-host"))]
pub fn decode_lowercase_hex(value: &str) -> Option<Vec<u8>> {
    decode_hex_pairs(value, false)
}

/// Decode even-length ASCII hexadecimal with either letter case.
#[cfg(any(test, feature = "icrc-host"))]
pub fn decode_hex(value: &str) -> Option<Vec<u8>> {
    decode_hex_pairs(value, true)
}

#[cfg(any(
    test,
    feature = "icrc-host",
    feature = "sns-host",
    feature = "subnet-catalog-host"
))]
fn decode_hex_pairs(value: &str, allow_uppercase: bool) -> Option<Vec<u8>> {
    let (pairs, remainder) = value.as_bytes().as_chunks::<2>();
    if !remainder.is_empty() {
        return None;
    }
    let nibble = |byte| match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' if allow_uppercase => Some(byte - b'A' + 10),
        _ => None,
    };
    pairs
        .iter()
        .map(|pair| Some(nibble(pair[0])? * 16 + nibble(pair[1])?))
        .collect()
}

///
/// HexEvidenceError
///
/// Shape failure for nonempty, byte-counted lowercase hexadecimal evidence.
///

#[cfg(any(feature = "cmc-host", feature = "certified-subnet-catalog-host"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HexEvidenceError {
    Empty,
    ByteCountOverflow,
    LengthMismatch,
    NonLowercaseHex,
}

/// Admit encoded evidence without decoding or allocating its bytes.
#[cfg(any(feature = "cmc-host", feature = "certified-subnet-catalog-host"))]
pub fn validate_nonempty_hex_evidence(
    value: &str,
    byte_count: usize,
) -> Result<(), HexEvidenceError> {
    if byte_count == 0 {
        return Err(HexEvidenceError::Empty);
    }
    let expected_length = byte_count
        .checked_mul(2)
        .ok_or(HexEvidenceError::ByteCountOverflow)?;
    if value.len() != expected_length {
        return Err(HexEvidenceError::LengthMismatch);
    }
    if !is_lowercase_hex(value) {
        return Err(HexEvidenceError::NonLowercaseHex);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(any(feature = "cmc-host", feature = "certified-subnet-catalog-host"))]
    use HexEvidenceError::{ByteCountOverflow, Empty, LengthMismatch, NonLowercaseHex};

    #[test]
    #[cfg(any(feature = "cmc-host", feature = "certified-subnet-catalog-host"))]
    fn encoded_evidence_requires_consistent_nonempty_bytes() {
        for (value, bytes, expected) in [
            ("", 0, Err(Empty)),
            ("zz", 0, Err(Empty)),
            ("", usize::MAX, Err(ByteCountOverflow)),
            ("", 1, Err(LengthMismatch)),
            ("0", 1, Err(LengthMismatch)),
            ("0A", 1, Err(NonLowercaseHex)),
            ("🦀", 2, Err(NonLowercaseHex)),
            ("ab", 1, Ok(())),
            ("00ff", 2, Ok(())),
        ] {
            assert_eq!(validate_nonempty_hex_evidence(value, bytes), expected);
        }
    }

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

    #[test]
    fn fixed_width_hex_requires_exactly_32_lowercase_bytes() {
        assert!(is_lowercase_hex_32_bytes(&"0a".repeat(32)));
        for value in [
            String::new(),
            "a".repeat(63),
            "a".repeat(65),
            "AA".repeat(32),
            "é".repeat(32),
        ] {
            assert!(!is_lowercase_hex_32_bytes(&value));
        }
    }

    #[test]
    fn lowercase_hex_decoder_accepts_complete_bytes_only() {
        assert_eq!(decode_lowercase_hex(""), Some(Vec::new()));
        assert_eq!(decode_lowercase_hex("000aff"), Some(vec![0, 10, 255]));
        assert_eq!(decode_lowercase_hex("0"), None);
        assert_eq!(decode_lowercase_hex("0A"), None);
        assert_eq!(decode_lowercase_hex("0g"), None);
        assert_eq!(decode_lowercase_hex("aé0"), None);
        assert_eq!(decode_lowercase_hex("🦀"), None);
        assert_eq!(decode_lowercase_hex("+a"), None);
        assert_eq!(decode_lowercase_hex(" 0a"), None);
    }

    #[test]
    fn hex_decoders_preserve_every_byte_and_their_case_rules() {
        let bytes = (u8::MIN..=u8::MAX).collect::<Vec<_>>();
        let lower = hex_bytes(&bytes);
        assert_eq!(decode_lowercase_hex(&lower), Some(bytes.clone()));
        assert_eq!(decode_hex(&lower), Some(bytes.clone()));
        assert_eq!(decode_hex(&lower.to_ascii_uppercase()), Some(bytes));
        assert_eq!(decode_hex("aBfF"), Some(vec![171, 255]));
        assert_eq!(decode_hex(""), Some(Vec::new()));
        for invalid in ["0", "0g", "aé0", "🦀", "+a", " 0a"] {
            assert_eq!(decode_hex(invalid), None, "{invalid:?}");
        }
    }
}
