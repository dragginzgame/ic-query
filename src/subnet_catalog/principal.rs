use super::CatalogError;
use candid::Principal;

/// Parse a textual IC principal into canonical text.
pub fn canonical_principal_text(value: &str) -> Result<String, CatalogError> {
    Ok(parse_principal(value, "principal")?.to_text())
}

pub fn parse_principal(value: &str, field: &'static str) -> Result<Principal, CatalogError> {
    Principal::from_text(value).map_err(|err| CatalogError::InvalidPrincipal {
        field,
        value: value.to_string(),
        reason: err.to_string(),
    })
}

pub fn principal_bytes(value: &str, field: &'static str) -> Result<Vec<u8>, CatalogError> {
    Ok(parse_principal(value, field)?.as_slice().to_vec())
}
