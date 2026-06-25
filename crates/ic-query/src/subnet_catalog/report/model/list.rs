//! Module: subnet_catalog::report::model::list
//!
//! Responsibility: define typed inputs and outputs for subnet catalog list reports.
//!
//! Does not own: catalog loading, row rendering, resolver behavior, or clap parsing.
//!
//! Boundary: keeps list report shape stable for text and JSON renderers while cache
//! policy remains in host modules.

use crate::subnet_catalog::{
    ClassificationSource, GeographicScope, RoutingRange, SubnetCatalogCacheRequest, SubnetKind,
    SubnetSpecialization,
};
use serde::{Deserialize, Serialize};

/// Optional classification filters applied to subnet catalog list reports.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SubnetCatalogFilters {
    pub kind: Option<SubnetKind>,
    pub specialization: Option<SubnetSpecialization>,
    pub geographic_scope: Option<GeographicScope>,
}

/// Inputs needed to build a subnet catalog list report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCatalogListRequest {
    pub cache: SubnetCatalogCacheRequest,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub stale_after_seconds: u64,
    pub filters: SubnetCatalogFilters,
    pub show_ranges: bool,
    pub range_limit: usize,
    pub range_offset: usize,
}

/// Serializable subnet catalog list report.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetCatalogListReport {
    pub schema_version: u32,
    pub network: String,
    pub catalog_path: String,
    pub catalog_schema_version: u32,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub catalog_stale: bool,
    pub stale_reason: String,
    pub resolver_backend: String,
    pub subnets: Vec<SubnetCatalogSubnetRow>,
}

/// One subnet row in a list report, including optional routing-range excerpts.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetCatalogSubnetRow {
    pub subnet_principal: String,
    pub subnet_kind: SubnetKind,
    pub subnet_kind_source: ClassificationSource,
    pub subnet_specialization: SubnetSpecialization,
    pub subnet_specialization_source: ClassificationSource,
    pub geographic_scope: GeographicScope,
    pub geographic_scope_source: ClassificationSource,
    pub subnet_label: String,
    pub subnet_label_source: ClassificationSource,
    pub node_count: Option<u32>,
    pub charges_apply_by_default: bool,
    pub range_count: usize,
    pub ranges_shown: usize,
    pub range_offset: usize,
    pub range_limit: usize,
    pub ranges: Vec<RoutingRange>,
}
