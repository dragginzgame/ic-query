//! Module: subnet_catalog::report::model::info
//!
//! Responsibility: define typed inputs and outputs for subnet catalog info reports.
//!
//! Does not own: subject resolution, cache refresh behavior, text rendering, or CLI parsing.
//!
//! Boundary: keeps the resolved-subject report contract explicit for both text and
//! JSON output.

use crate::subnet_catalog::{
    CacheDisposition, CatalogAssurance, CatalogReadPolicy, CatalogSourceSelection,
    ClassificationSource, GeographicScope, ResolveAs, RoutingRange, SubnetCatalogCacheRequest,
    SubnetKind, SubnetSpecialization,
};
use serde::{Deserialize, Serialize};

///
/// SubnetCatalogInfoRequest
///
/// Inputs needed to resolve one subnet catalog subject and build an info report.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCatalogInfoRequest {
    pub cache: SubnetCatalogCacheRequest,
    /// Exact cache and network behavior authorized for this report.
    pub read_policy: CatalogReadPolicy,
    pub input: String,
    pub forced: Option<ResolveAs>,
    pub now_unix_secs: u64,
    pub stale_after_seconds: u64,
}

impl SubnetCatalogInfoRequest {
    #[must_use]
    pub fn new(
        cache: SubnetCatalogCacheRequest,
        source_endpoint: impl Into<String>,
        input: impl Into<String>,
        now_unix_secs: u64,
        stale_after_seconds: u64,
    ) -> Self {
        Self {
            cache,
            read_policy: CatalogReadPolicy::RefreshMissingOrInvalid {
                source: CatalogSourceSelection::uncertified_query(source_endpoint),
            },
            input: input.into(),
            forced: None,
            now_unix_secs,
            stale_after_seconds,
        }
    }

    #[must_use]
    pub const fn with_forced(mut self, forced: ResolveAs) -> Self {
        self.forced = Some(forced);
        self
    }

    #[must_use]
    pub fn with_read_policy(mut self, read_policy: CatalogReadPolicy) -> Self {
        self.read_policy = read_policy;
        self
    }
}

///
/// SubnetCatalogInfoReport
///
/// Serializable detail report for a subnet or canister subject.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetCatalogInfoReport {
    pub schema_version: u32,
    pub input_principal: String,
    pub resolved_as: String,
    pub resolved_from: String,
    pub subnet_principal: String,
    /// Raw numeric `SubnetType` discriminant from the Registry record.
    pub registry_subnet_type: i32,
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
    /// Assurance established for this exact snapshot.
    pub assurance: CatalogAssurance,
    /// Source endpoints contributing to the snapshot.
    pub source_endpoints: Vec<String>,
    /// Canonical Registry payload digest agreed by every source endpoint.
    pub agreement_digest: Option<String>,
    /// Exact number of Registry query calls made during collection.
    pub registry_query_call_count: u64,
    /// Lowercase SHA-256 digest of the canonical catalog payload.
    pub catalog_digest: String,
    /// Observable result of applying the requested cache policy.
    pub cache_disposition: CacheDisposition,
    pub catalog_schema_version: u32,
    pub catalog_path: String,
    pub fetched_at: String,
    pub catalog_stale: bool,
    pub stale_reason: String,
    pub resolver_backend: String,
    /// Collector package version recorded by the source.
    pub collector_version: String,
    /// Classification contract version.
    pub classification_schema_version: u32,
    /// Lowercase SHA-256 digest of the classification policy.
    pub classification_policy_digest: String,
    /// Resolver contract version.
    pub resolver_schema_version: u32,
    pub matched_canister_principal: Option<String>,
    pub matched_routing_range: Option<RoutingRange>,
    pub cycles_per_billion_instructions: Option<u128>,
    pub rate_source: Option<String>,
    pub formula_version: Option<String>,
}
