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
    /// Network and root directory identifying the managed catalog snapshot.
    pub cache: SubnetCatalogCacheRequest,
    /// Exact cache and network behavior authorized for this report.
    pub read_policy: CatalogReadPolicy,
    /// Exact canister or Subnet principal, or a unique Subnet principal prefix.
    pub input: String,
    /// Explicit subject interpretation, absent for automatic resolution.
    pub forced: Option<ResolveAs>,
    /// Caller-supplied observation time in Unix seconds.
    pub now_unix_secs: u64,
    /// Age threshold in seconds used to describe snapshot freshness.
    pub stale_after_seconds: u64,
}

impl SubnetCatalogInfoRequest {
    /// Create a subject lookup that refreshes a missing or invalid catalog through the selected endpoint.
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

    /// Require the supplied subject to be interpreted as the selected principal kind.
    #[must_use]
    pub const fn with_forced(mut self, forced: ResolveAs) -> Self {
        self.forced = Some(forced);
        self
    }

    /// Select the exact cache and network behavior authorized for this report.
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
    /// Version of this report contract; currently `1`.
    pub schema_version: u32,
    /// Canonical principal resolved from the supplied subject.
    pub input_principal: String,
    /// Whether resolution treated the subject as a Subnet or a canister.
    pub resolved_as: String,
    /// Resolution evidence label: a Subnet principal match or a routing range.
    pub resolved_from: String,
    /// Canonical principal of the Subnet described by this row.
    pub subnet_principal: String,
    /// Raw numeric `SubnetType` discriminant from the Registry record.
    pub registry_subnet_type: i32,
    /// IC-native classification derived from the Registry Subnet type code.
    pub subnet_kind: SubnetKind,
    /// Authority that supplied the Subnet kind classification.
    pub subnet_kind_source: ClassificationSource,
    /// Curated or default Subnet specialization.
    pub subnet_specialization: SubnetSpecialization,
    /// Authority that supplied the specialization classification.
    pub subnet_specialization_source: ClassificationSource,
    /// Curated or default geographic classification of the Subnet.
    pub geographic_scope: GeographicScope,
    /// Authority that supplied the geographic classification.
    pub geographic_scope_source: ClassificationSource,
    /// Human-facing label assigned by the current classification policy.
    pub subnet_label: String,
    /// Authority that supplied the human-facing Subnet label.
    pub subnet_label_source: ClassificationSource,
    /// Node membership count, when supplied by the Registry Subnet record.
    pub node_count: Option<u32>,
    /// Whether application charging applies to the resolved subject.
    pub charges_apply_to_subject: bool,
    /// Explanation of the charging decision for this subject.
    pub charge_applicability_reason: String,
    /// Canonical Registry canister principal supplying the snapshot.
    pub registry_canister_id: String,
    /// Exact Registry version represented by the catalog.
    pub registry_version: u64,
    /// Assurance established for this exact snapshot.
    pub assurance: CatalogAssurance,
    /// Source endpoints contributing to the snapshot.
    pub source_endpoints: Vec<String>,
    /// Canonical Registry payload digest agreed by every source endpoint.
    pub agreement_digest: Option<String>,
    /// Exact number of Registry query calls made during collection.
    pub registry_query_call_count: u64,
    /// Registry record family selected as routing authority.
    pub routing_source: crate::subnet_catalog::SubnetCatalogRoutingSource,
    /// Per-value Registry provenance retained by the catalog.
    pub registry_records: Vec<crate::subnet_catalog::SubnetCatalogRegistryRecordEvidence>,
    /// Lowercase SHA-256 digest of the canonical catalog payload.
    pub catalog_digest: String,
    /// Observable result of applying the requested cache policy.
    pub cache_disposition: CacheDisposition,
    /// Persisted catalog schema identifier; currently `1`.
    pub catalog_schema_version: u32,
    /// Managed snapshot path selected by the cache request.
    pub catalog_path: String,
    /// UTC collection timestamp retained in catalog provenance.
    pub fetched_at: String,
    /// Whether the snapshot age exceeds policy or its timestamp is unusable.
    pub catalog_stale: bool,
    /// Machine-readable reason for the reported freshness decision.
    pub stale_reason: String,
    /// Implementation identity of the recorded routing resolver policy.
    pub resolver_backend: String,
    /// Collector package version recorded by the source.
    pub collector_version: String,
    /// Classification contract version.
    pub classification_schema_version: u32,
    /// Lowercase SHA-256 digest of the classification policy.
    pub classification_policy_digest: String,
    /// Resolver contract version.
    pub resolver_schema_version: u32,
    /// Exact canonical canister principal for a canister subject; absent for a Subnet subject.
    pub matched_canister_principal: Option<String>,
    /// Inclusive routing authority range used for a canister subject.
    pub matched_routing_range: Option<RoutingRange>,
    /// Application execution rate in cycles per billion instructions, when applicable.
    pub cycles_per_billion_instructions: Option<u128>,
    /// Authority used for the reported execution rate, when one applies.
    pub rate_source: Option<String>,
    /// Execution-rate formula identifier, when one applies.
    pub formula_version: Option<String>,
}
