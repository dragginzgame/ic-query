//! Module: subnet_catalog::json
//!
//! Responsibility: decode, structurally validate, and encode raw subnet catalog JSON payloads.
//!
//! Does not own: cache paths, catalog fetching, or human text rendering.
//!
//! Boundary: parsed values remain untrusted raw evidence; authority validation is separate.

use super::{CatalogError, RawSubnetCatalog};

/// Decode and structurally validate one untrusted raw subnet catalog JSON payload.
pub fn parse_catalog_json(data: &str) -> Result<RawSubnetCatalog, CatalogError> {
    let catalog = serde_json::from_str::<RawSubnetCatalog>(data)?;
    catalog.validate()?;
    Ok(catalog)
}

/// Renders one subnet catalog JSON payload with stable pretty formatting.
pub fn catalog_to_pretty_json(catalog: &RawSubnetCatalog) -> Result<String, CatalogError> {
    Ok(serde_json::to_string_pretty(catalog)?)
}
