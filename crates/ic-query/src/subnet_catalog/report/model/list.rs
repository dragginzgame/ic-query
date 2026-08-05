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
    pub kind: Option<SubnetKind>,
    pub specialization: Option<SubnetSpecialization>,
    pub geographic_scope: Option<GeographicScope>,
}

impl SubnetCatalogFilters {
    #[must_use]
    pub const fn with_kind(mut self, kind: SubnetKind) -> Self {
        self.kind = Some(kind);
        self
    }

    #[must_use]
    pub const fn with_specialization(mut self, specialization: SubnetSpecialization) -> Self {
        self.specialization = Some(specialization);
        self
    }

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
    pub cache: SubnetCatalogCacheRequest,
    /// Exact cache and network behavior authorized for this report.
    pub read_policy: CatalogReadPolicy,
    pub now_unix_secs: u64,
    pub stale_after_seconds: u64,
    pub filters: SubnetCatalogFilters,
    pub show_ranges: bool,
    pub range_limit: usize,
    pub range_offset: usize,
}

impl SubnetCatalogListRequest {
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

    #[must_use]
    pub fn with_read_policy(mut self, read_policy: CatalogReadPolicy) -> Self {
        self.read_policy = read_policy;
        self
    }

    #[must_use]
    pub const fn with_filters(mut self, filters: SubnetCatalogFilters) -> Self {
        self.filters = filters;
        self
    }

    #[must_use]
    pub const fn with_kind(mut self, kind: SubnetKind) -> Self {
        self.filters.kind = Some(kind);
        self
    }

    #[must_use]
    pub const fn with_specialization(mut self, specialization: SubnetSpecialization) -> Self {
        self.filters.specialization = Some(specialization);
        self
    }

    #[must_use]
    pub const fn with_geographic_scope(mut self, geographic_scope: GeographicScope) -> Self {
        self.filters.geographic_scope = Some(geographic_scope);
        self
    }

    #[must_use]
    pub const fn with_show_ranges(mut self, show_ranges: bool) -> Self {
        self.show_ranges = show_ranges;
        self
    }

    #[must_use]
    pub const fn with_range_limit(mut self, range_limit: usize) -> Self {
        self.range_limit = range_limit;
        self
    }

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
    pub schema_version: u32,
    pub network: String,
    pub catalog_path: String,
    pub catalog_schema_version: u32,
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
    pub subnets: Vec<SubnetCatalogSubnetRow>,
}

///
/// SubnetCatalogSubnetRow
///
/// One subnet row in a list report, including optional routing-range excerpts.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetCatalogSubnetRow {
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
    pub charges_apply_by_default: bool,
    pub range_count: usize,
    pub ranges_shown: usize,
    pub range_offset: usize,
    pub range_limit: usize,
    pub ranges: Vec<RoutingRange>,
}
