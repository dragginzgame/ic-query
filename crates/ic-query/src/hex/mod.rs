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
#[cfg(any(
    test,
    feature = "dashboard-host",
    feature = "nns-host",
    feature = "sns-host",
    feature = "cmc-host"
))]
pub fn is_lowercase_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

#[must_use]
#[cfg(any(
    test,
    feature = "dashboard-host",
    feature = "nns-host",
    feature = "sns-host"
))]
pub fn is_lowercase_hex_32_bytes(value: &str) -> bool {
    value.len() == 64 && is_lowercase_hex(value)
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

#[cfg(any(feature = "cmc-host", feature = "nns-host"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HexEvidenceError {
    Empty,
    ByteCountOverflow,
    LengthMismatch,
    NonLowercaseHex,
}

/// Admit encoded evidence without decoding or allocating its bytes.
#[cfg(any(feature = "cmc-host", feature = "nns-host"))]
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
mod tests;
