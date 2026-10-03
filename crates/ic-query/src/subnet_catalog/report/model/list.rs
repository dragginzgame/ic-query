//! Module: subnet_catalog::report::model::list
//!
//! Responsibility: define typed inputs and outputs for subnet catalog list reports.
//!
//! Does not own: catalog loading, row rendering, resolver behavior, or clap parsing.
//!
//! Boundary: keeps list report shape stable for text and JSON renderers while cache
//! policy remains in host modules.

use crate::subnet_catalog::{
    CacheDisposition, CatalogAssurance, CatalogReadPolicy, CatalogSourceSelection,
    ClassificationSource, GeographicScope, RoutingRange, SubnetCatalogCacheRequest, SubnetKind,
    SubnetSpecialization,
};
use serde::{Deserialize, Serialize};

///
/// SubnetCatalogFilters
///
/// Optional classification filters applied to subnet catalog list reports.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SubnetCatalogFilters {
    /// Optional exact IC-native Subnet kind filter.
    pub kind: Option<SubnetKind>,
    /// Optional exact curated specialization filter.
    pub specialization: Option<SubnetSpecialization>,
    /// Optional exact Subnet geographic classification filter.
    pub geographic_scope: Option<GeographicScope>,
}

impl SubnetCatalogFilters {
    /// Require the selected IC-native Subnet kind.
    #[must_use]
    pub const fn with_kind(mut self, kind: SubnetKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Require the selected curated Subnet specialization.
    #[must_use]
    pub const fn with_specialization(mut self, specialization: SubnetSpecialization) -> Self {
        self.specialization = Some(specialization);
        self
    }

    /// Require the selected Subnet geographic classification.
    #[must_use]
    pub const fn with_geographic_scope(mut self, geographic_scope: GeographicScope) -> Self {
        self.geographic_scope = Some(geographic_scope);
        self
    }
}

///
/// SubnetCatalogListRequest
///
/// Inputs needed to build a subnet catalog list report.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubnetCatalogListRequest {
    /// Network and root directory identifying the managed catalog snapshot.
    pub cache: SubnetCatalogCacheRequest,
    /// Exact cache and network behavior authorized for this report.
    pub read_policy: CatalogReadPolicy,
    /// Caller-supplied observation time in Unix seconds.
    pub now_unix_secs: u64,
    /// Age threshold in seconds used to describe snapshot freshness.
    pub stale_after_seconds: u64,
    /// Classification filters applied to the complete catalog before rendering.
    pub filters: SubnetCatalogFilters,
    /// Whether the list view includes routing-range detail.
    pub show_ranges: bool,
    /// Maximum routing ranges shown per Subnet in this view.
    pub range_limit: usize,
    /// Number of canonical routing ranges skipped per Subnet in this view.
    pub range_offset: usize,
}

impl SubnetCatalogListRequest {
    /// Create an unfiltered list view that refreshes a missing or invalid catalog through the selected endpoint.
    #[must_use]
    pub fn new(
        cache: SubnetCatalogCacheRequest,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        stale_after_seconds: u64,
    ) -> Self {
        Self {
            cache,
            read_policy: CatalogReadPolicy::RefreshMissingOrInvalid {
                source: CatalogSourceSelection::uncertified_query(source_endpoint),
            },
            now_unix_secs,
            stale_after_seconds,
            filters: SubnetCatalogFilters::default(),
            show_ranges: false,
            range_limit: 50,
            range_offset: 0,
        }
    }

    /// Select the exact cache and network behavior authorized for this report.
    #[must_use]
    pub fn with_read_policy(mut self, read_policy: CatalogReadPolicy) -> Self {
        self.read_policy = read_policy;
        self
    }

    /// Replace all classification filters used for this list view.
    #[must_use]
    pub const fn with_filters(mut self, filters: SubnetCatalogFilters) -> Self {
        self.filters = filters;
        self
    }

    /// Require the selected IC-native Subnet kind.
    #[must_use]
    pub const fn with_kind(mut self, kind: SubnetKind) -> Self {
        self.filters.kind = Some(kind);
        self
    }

    /// Require the selected curated Subnet specialization.
    #[must_use]
    pub const fn with_specialization(mut self, specialization: SubnetSpecialization) -> Self {
        self.filters.specialization = Some(specialization);
        self
    }

    /// Require the selected Subnet geographic classification.
    #[must_use]
    pub const fn with_geographic_scope(mut self, geographic_scope: GeographicScope) -> Self {
        self.filters.geographic_scope = Some(geographic_scope);
        self
    }

    /// Include or omit routing-range detail in the list view.
    #[must_use]
    pub const fn with_show_ranges(mut self, show_ranges: bool) -> Self {
        self.show_ranges = show_ranges;
        self
    }

    /// Limit the routing ranges shown per Subnet without changing the cached snapshot.
    #[must_use]
    pub const fn with_range_limit(mut self, range_limit: usize) -> Self {
        self.range_limit = range_limit;
        self
    }

    /// Skip routing ranges per Subnet without changing the cached snapshot.
    #[must_use]
    pub const fn with_range_offset(mut self, range_offset: usize) -> Self {
        self.range_offset = range_offset;
        self
    }
}

///
/// SubnetCatalogListReport
///
/// Serializable subnet catalog list report.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetCatalogListReport {
    /// Version of this report contract; currently `1`.
    pub schema_version: u32,
    /// Network identity of the catalog; current mainnet reports use `ic`.
    pub network: String,
    /// Managed snapshot path selected by the cache request.
    pub catalog_path: String,
    /// Persisted catalog schema identifier; currently `1`.
    pub catalog_schema_version: u32,
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
    /// Filtered Subnet rows in canonical catalog order.
    pub subnets: Vec<SubnetCatalogSubnetRow>,
}

///
/// SubnetCatalogSubnetRow
///
/// One subnet row in a list report, including optional routing-range excerpts.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetCatalogSubnetRow {
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
    /// Whether application charging applies to this Subnet kind by default.
    pub charges_apply_by_default: bool,
    /// Total routing ranges assigned to this Subnet in the complete catalog.
    pub range_count: usize,
    /// Number of routing ranges included in this row's view.
    pub ranges_shown: usize,
    /// Number of canonical routing ranges skipped per Subnet in this view.
    pub range_offset: usize,
    /// Maximum routing ranges shown per Subnet in this view.
    pub range_limit: usize,
    /// Selected inclusive routing ranges assigned to this Subnet.
    pub ranges: Vec<RoutingRange>,
}
