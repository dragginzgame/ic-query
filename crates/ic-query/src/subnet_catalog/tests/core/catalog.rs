use super::fixtures::fixture_catalog;
use crate::subnet_catalog::{CATALOG_SCHEMA_VERSION, CatalogError, parse_catalog_json};

#[test]
fn catalog_schema_round_trips_through_json() {
    let catalog = fixture_catalog();
    let json = serde_json::to_string_pretty(&catalog).expect("serialize catalog");
    let decoded = parse_catalog_json(&json).expect("parse catalog");

    assert_eq!(decoded, catalog);
}

#[test]
fn unknown_future_schema_version_is_rejected() {
    let mut catalog = fixture_catalog();
    catalog.catalog_schema_version = CATALOG_SCHEMA_VERSION + 1;
    let json = serde_json::to_string(&catalog).expect("serialize catalog");

    let err = parse_catalog_json(&json).expect_err("future schema must fail");

    assert!(matches!(
        err,
        CatalogError::UnsupportedSchemaVersion {
            found,
            supported: CATALOG_SCHEMA_VERSION
        } if found == CATALOG_SCHEMA_VERSION + 1
    ));
}
