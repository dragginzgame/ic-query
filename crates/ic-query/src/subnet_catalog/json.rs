//! Module: subnet_catalog::json
//!
//! Responsibility: decode, validate, and encode subnet catalog JSON payloads.
//!
//! Does not own: cache paths, catalog fetching, or human text rendering.
//!
//! Boundary: keeps wire/file JSON conversion centralized so callers work with
//! validated domain structs.

use super::{CatalogError, SubnetCatalog};

/// Decodes and validates one subnet catalog JSON payload.
pub fn parse_catalog_json(data: &str) -> Result<SubnetCatalog, CatalogError> {
    let catalog = serde_json::from_str::<SubnetCatalog>(data)?;
    catalog.validate()?;
    Ok(catalog)
}

/// Renders one subnet catalog JSON payload with stable pretty formatting.
pub fn catalog_to_pretty_json(catalog: &SubnetCatalog) -> Result<String, CatalogError> {
    Ok(serde_json::to_string_pretty(catalog)?)
}
