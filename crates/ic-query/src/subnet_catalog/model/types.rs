//! Module: subnet_catalog::model::types
//!
//! Responsibility: define the persisted subnet catalog domain records.
//!
//! Does not own: catalog validation, host cache paths, report shaping, or CLI filters.
//!
//! Boundary: keeps raw catalog identity and subnet/routing data separate from
//! human-facing report rows and resolver outputs.

use super::{ClassificationSource, GeographicScope, SubnetKind, SubnetSpecialization};
use serde::{Deserialize, Serialize};

/// Persisted subnet catalog snapshot loaded from or written to the local cache.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetCatalog {
    pub catalog_schema_version: u32,
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub resolver_backend: String,
    pub subnets: Vec<SubnetInfo>,
    pub routing_ranges: Vec<RoutingRange>,
}

/// One subnet entry and its classification metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetInfo {
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
}

/// Inclusive canister routing range assigned to one subnet.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoutingRange {
    pub start_canister_id: String,
    pub end_canister_id: String,
    pub subnet_principal: String,
}
