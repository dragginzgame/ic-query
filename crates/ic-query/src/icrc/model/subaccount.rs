//! Module: icrc::model::subaccount
//!
//! Responsibility: validate and normalize ICRC subaccount hex values.
//! Does not own: command parsing, account construction, or report fields.
//! Boundary: preserves absent values or produces exactly 32 validated bytes and canonical hex.

use super::IcrcError;
use crate::hex::{decode_lowercase_hex, hex_bytes};

/// Validates and normalizes a 32-byte ICRC subaccount hex string.
pub fn normalize_subaccount_hex(value: &str) -> Result<String, IcrcError> {
    let bytes = subaccount_bytes_from_hex(value)?;
    Ok(hex_bytes(&bytes))
}

#[cfg(feature = "icrc-host")]
pub(in crate::icrc) fn normalize_optional_subaccount_hex(
    value: Option<&str>,
) -> Result<Option<String>, IcrcError> {
    value.map(normalize_subaccount_hex).transpose()
}

pub(in crate::icrc) fn subaccount_bytes_from_hex(value: &str) -> Result<Vec<u8>, IcrcError> {
    let value = value.trim();
    if !value.len().is_multiple_of(2) {
        return Err(IcrcError::InvalidSubaccountHex {
            reason: "hex string must contain an even number of characters".to_string(),
        });
    }
    let bytes = decode_lowercase_hex(&value.to_ascii_lowercase()).ok_or_else(|| {
        IcrcError::InvalidSubaccountHex {
            reason: "hex string must contain only ASCII hexadecimal digits".to_string(),
        }
    })?;
    if bytes.len() != 32 {
        return Err(IcrcError::InvalidSubaccountLength { bytes: bytes.len() });
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_subaccounts_return_typed_hex_errors() {
        for value in [
            format!("aé{}", "0".repeat(61)),
            "🦀".repeat(16),
            "+a".repeat(32),
            "gg".repeat(32),
            "a".repeat(63),
        ] {
            assert!(
                matches!(
                    normalize_subaccount_hex(&value),
                    Err(IcrcError::InvalidSubaccountHex { .. })
                ),
                "malformed subaccount {value:?}",
            );
        }
    }

    #[test]
    fn subaccounts_normalize_case_and_whitespace_and_require_32_bytes() {
        let mixed_case = "aB01".repeat(16);
        assert_eq!(
            normalize_subaccount_hex(&format!(" \t{mixed_case}\n")).expect("valid subaccount"),
            "ab01".repeat(16),
        );
        for bytes in [0, 31, 33] {
            assert!(matches!(
                normalize_subaccount_hex(&"ab".repeat(bytes)),
                Err(IcrcError::InvalidSubaccountLength { bytes: actual }) if actual == bytes,
            ));
        }
    }
}
