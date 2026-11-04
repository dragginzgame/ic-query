use super::*;
#[cfg(any(feature = "cmc-host", feature = "nns-host"))]
use HexEvidenceError::{ByteCountOverflow, Empty, LengthMismatch, NonLowercaseHex};

#[test]
#[cfg(any(feature = "cmc-host", feature = "nns-host"))]
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
