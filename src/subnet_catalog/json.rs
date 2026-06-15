use super::{CatalogError, SubnetCatalog};

/// Decode and validate one subnet catalog JSON payload.
pub fn parse_catalog_json(data: &str) -> Result<SubnetCatalog, CatalogError> {
    let catalog = serde_json::from_str::<SubnetCatalog>(data)?;
    catalog.validate()?;
    Ok(catalog)
}

/// Render one subnet catalog JSON payload with stable pretty formatting.
pub fn catalog_to_pretty_json(catalog: &SubnetCatalog) -> Result<String, CatalogError> {
    Ok(serde_json::to_string_pretty(catalog)?)
}
