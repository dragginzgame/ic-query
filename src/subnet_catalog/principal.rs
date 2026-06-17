//! Module: subnet_catalog::principal
//!
//! Responsibility: parse and normalize principal text used by subnet catalog records.
//!
//! Does not own: subject resolution, CLI argument parsing, or compact display text.
//!
//! Boundary: converts caller-provided principal text into canonical Candid principal
//! forms and typed catalog errors.

use super::CatalogError;
use candid::Principal;

/// Parses a textual IC principal into canonical text.
pub fn canonical_principal_text(value: &str) -> Result<String, CatalogError> {
    Ok(parse_principal(value, "principal")?.to_text())
}

/// Parses a textual IC principal and labels validation errors with the source field.
pub fn parse_principal(value: &str, field: &'static str) -> Result<Principal, CatalogError> {
    Principal::from_text(value).map_err(|err| CatalogError::InvalidPrincipal {
        field,
        value: value.to_string(),
        reason: err.to_string(),
    })
}

/// Parses a textual IC principal into its raw byte representation.
pub fn principal_bytes(value: &str, field: &'static str) -> Result<Vec<u8>, CatalogError> {
    Ok(parse_principal(value, field)?.as_slice().to_vec())
}
