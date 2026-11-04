//! Module: subnet_catalog::report::model::info
//!
//! Responsibility: define typed inputs and outputs for subnet catalog info reports.
//!
//! Does not own: subject resolution, cache refresh behavior, text rendering, or CLI parsing.
//!
//! Boundary: keeps the resolved-subject report contract explicit for both text and
//! JSON output.

use crate::subnet_catalog::{
    CacheDisposition, CatalogReadPolicy, CatalogSourceSelection, ResolveAs, RoutingRange,
    SubnetCatalogCacheRequest,
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
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub classification: crate::subnet_catalog::SubnetClassification,
    /// Node membership count, when supplied by the Registry Subnet record.
    pub node_count: Option<u32>,
    /// Whether application charging applies to the resolved subject.
    pub charges_apply_to_subject: bool,
    /// Explanation of the charging decision for this subject.
    pub charge_applicability_reason: String,
    /// Observable result of applying the requested cache policy.
    pub cache_disposition: CacheDisposition,
    /// Persisted catalog schema identifier; currently `1`.
    pub catalog_schema_version: u32,
    /// Managed snapshot path selected by the cache request.
    pub catalog_path: String,
    /// Whether the snapshot age exceeds policy or its timestamp is unusable.
    pub catalog_stale: bool,
    /// Machine-readable reason for the reported freshness decision.
    pub stale_reason: String,
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
    /// Collection and policy evidence from the admitted catalog.
    #[serde(flatten)]
    pub metadata: super::SubnetCatalogReportMetadata,
}
