//! Module: subnet_catalog::report::model::info
//!
//! Responsibility: define typed inputs and outputs for subnet catalog info reports.
//!
//! Does not own: subject resolution, cache refresh behavior, text rendering, or CLI parsing.
//!
//! Boundary: keeps the resolved-subject report contract explicit for both text and
//! JSON output.

use crate::subnet_catalog::{
    ClassificationSource, GeographicScope, ResolveAs, RoutingRange, SubnetCatalogCacheRequest,
    SubnetKind, SubnetSpecialization,
};
use serde::{Deserialize, Serialize};

/// Inputs needed to resolve one subnet catalog subject and build an info report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCatalogInfoRequest {
    pub cache: SubnetCatalogCacheRequest,
    pub source_endpoint: String,
    pub input: String,
    pub forced: Option<ResolveAs>,
    pub now_unix_secs: u64,
    pub stale_after_seconds: u64,
}

/// Serializable detail report for a subnet or canister subject.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetCatalogInfoReport {
    pub schema_version: u32,
    pub input_principal: String,
    pub resolved_as: String,
    pub resolved_from: String,
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
    pub charges_apply_to_subject: bool,
    pub charge_applicability_reason: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub catalog_schema_version: u32,
    pub catalog_path: String,
    pub fetched_at: String,
    pub catalog_stale: bool,
    pub stale_reason: String,
    pub resolver_backend: String,
    pub matched_canister_principal: Option<String>,
    pub matched_routing_range: Option<RoutingRange>,
    pub cycles_per_billion_instructions: Option<u128>,
    pub rate_source: Option<String>,
    pub formula_version: Option<String>,
}
