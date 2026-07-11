//! Module: icrc::model::subaccount
//!
//! Responsibility: validate and normalize ICRC subaccount hex values.
//! Does not own: command parsing, account construction, or report fields.
//! Boundary: produces exactly 32 validated bytes or a typed ICRC error.

use super::IcrcError;
use crate::hex::hex_bytes;

/// Validates and normalizes a 32-byte ICRC subaccount hex string.
pub fn normalize_subaccount_hex(value: &str) -> Result<String, IcrcError> {
    let bytes = subaccount_bytes_from_hex(value)?;
    Ok(hex_bytes(&bytes))
}

pub(in crate::icrc) fn subaccount_bytes_from_hex(value: &str) -> Result<Vec<u8>, IcrcError> {
    let value = value.trim();
    if !value.len().is_multiple_of(2) {
        return Err(IcrcError::InvalidSubaccountHex {
            reason: "hex string must contain an even number of characters".to_string(),
        });
    }
    let bytes = (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16).map_err(|err| {
                IcrcError::InvalidSubaccountHex {
                    reason: err.to_string(),
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if bytes.len() != 32 {
        return Err(IcrcError::InvalidSubaccountLength { bytes: bytes.len() });
    }
    Ok(bytes)
}
