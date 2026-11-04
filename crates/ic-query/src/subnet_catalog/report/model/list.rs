//! Module: subnet_catalog::report::model::list
//!
//! Responsibility: define typed inputs and outputs for subnet catalog list reports.
//!
//! Does not own: catalog loading, row rendering, resolver behavior, or clap parsing.
//!
//! Boundary: keeps list report shape stable for text and JSON renderers while cache
//! policy remains in host modules.

use crate::subnet_catalog::{
    CacheDisposition, CatalogReadPolicy, CatalogSourceSelection, GeographicScope, RoutingRange,
    SubnetCatalogCacheRequest, SubnetKind, SubnetSpecialization,
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
    /// Observable result of applying the requested cache policy.
    pub cache_disposition: CacheDisposition,
    /// Whether the snapshot age exceeds policy or its timestamp is unusable.
    pub catalog_stale: bool,
    /// Machine-readable reason for the reported freshness decision.
    pub stale_reason: String,
    /// Filtered Subnet rows in canonical catalog order.
    pub subnets: Vec<SubnetCatalogSubnetRow>,
    /// Collection and policy evidence from the admitted catalog.
    #[serde(flatten)]
    pub metadata: super::SubnetCatalogReportMetadata,
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
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub classification: crate::subnet_catalog::SubnetClassification,
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
