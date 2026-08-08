//! Module: cloud_engine::model
//!
//! Responsibility: define stable CloudEngine report and source-data models.
//! Does not own: live transport, source validation, CLI parsing, or text rendering.
//! Boundary: preserves raw control-plane identities, prices, timestamps, and authority limits.

use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(feature = "cloud-engine-host")]
use super::CloudEngineSourceRequest;
#[cfg(all(feature = "cloud-engine-host", feature = "subnet-catalog-host"))]
use crate::subnet_catalog::{
    CacheDisposition, CatalogAssurance, ClassificationSource, GeographicScope, SubnetKind,
    SubnetSpecialization,
};

///
/// CloudEngineReportContext
///
/// Provenance shared by direct CloudEngine control-plane reports.
///

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct CloudEngineReportContext {
    /// Report schema version.
    pub schema_version: u32,
    /// Queried network identity.
    pub network: String,
    /// Authority represented by the report.
    pub authority: String,
    /// CloudEngine control-plane registry canister principal.
    pub engine_canister_id: String,
    /// UTC collection timestamp.
    pub fetched_at: String,
    /// Replica endpoint used for the queries.
    pub source_endpoint: String,
    /// Collector identity.
    pub fetched_by: String,
    /// Whether the application data was cryptographically certified.
    pub certified: bool,
    /// Whether sequential calls form one point-in-time view.
    pub point_in_time_guaranteed: bool,
    /// Number of native canister query calls represented by the report.
    pub query_call_count: usize,
}

///
/// CloudEngineOperatorReport
///
/// Public operator binding and settings for one CloudEngine Subnet.
///

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct CloudEngineOperatorReport {
    /// Query and authority provenance.
    #[serde(flatten)]
    pub context: CloudEngineReportContext,
    /// Canonical requested Subnet principal.
    pub subnet_id: String,
    /// Whether the control-plane registry returned an operator binding.
    pub operator_binding_present: bool,
    /// Per-engine operator canister principal when registered.
    pub operator_canister_id: Option<String>,
    /// Public engine-owner principal returned by the operator.
    pub engine_owner: Option<String>,
    /// Public platform-administrator principal returned by the operator.
    pub platform_admin: Option<String>,
    /// Public Caffeine integration setting; `None` means the setting is absent.
    pub caffeine_enabled: Option<bool>,
    /// Number of claimed domains, or `None` when the operator returned no domain field.
    pub claimed_domain_count: Option<usize>,
    /// Canonically ordered claimed custom-domain names.
    pub claimed_domains: Option<Vec<String>>,
}

///
/// CloudEngineNodeType
///
/// CloudEngine marketplace node class.
///

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum CloudEngineNodeType {
    /// Type 4.1 node class.
    #[serde(rename = "type4.1")]
    Type4_1,
    /// Type 4.2 node class.
    #[serde(rename = "type4.2")]
    Type4_2,
    /// Type 4.3 node class.
    #[serde(rename = "type4.3")]
    Type4_3,
    /// Type 4.4 node class.
    #[serde(rename = "type4.4")]
    Type4_4,
    /// Type 4.5 node class.
    #[serde(rename = "type4.5")]
    Type4_5,
}

impl CloudEngineNodeType {
    /// Stable CloudEngine marketplace label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Type4_1 => "type4.1",
            Self::Type4_2 => "type4.2",
            Self::Type4_3 => "type4.3",
            Self::Type4_4 => "type4.4",
            Self::Type4_5 => "type4.5",
        }
    }
}

impl fmt::Display for CloudEngineNodeType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

///
/// CloudEnginePriceRow
///
/// One public CloudEngine marketplace price override.
///

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct CloudEnginePriceRow {
    /// Canonical flattened marketplace key.
    pub key: String,
    /// Node class priced by this row.
    pub node_type: CloudEngineNodeType,
    /// Optional Registry data-center identifier for a location-specific override.
    pub data_center_id: Option<String>,
    /// Optional provider principal for a provider-specific override.
    pub provider_id: Option<String>,
    /// Provider share in raw cycles per month.
    pub net_cycles_per_month: String,
    /// Customer charge in raw cycles per month, including the network fee.
    pub gross_cycles_per_month: String,
    /// Raw control-plane update timestamp in Unix nanoseconds.
    pub updated_at_unix_nanos: i64,
}

///
/// CloudEnginePricesReport
///
/// Bounded public CloudEngine marketplace fee and price report.
///

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct CloudEnginePricesReport {
    /// Query and authority provenance.
    #[serde(flatten)]
    pub context: CloudEngineReportContext,
    /// Network-fee fraction added to provider net prices.
    pub network_fee: f64,
    /// Number of returned marketplace rows.
    pub price_count: usize,
    /// Canonically ordered marketplace prices.
    pub prices: Vec<CloudEnginePriceRow>,
}

///
/// CloudEngineOperatorLookupStatus
///
/// Outcome of one exact control-plane operator-binding lookup for a Registry Subnet.
///

#[cfg(all(feature = "cloud-engine-host", feature = "subnet-catalog-host"))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudEngineOperatorLookupStatus {
    /// The control plane returned an operator canister principal.
    Resolved,
    /// The control plane successfully returned no operator binding.
    Absent,
    /// The exact control-plane query failed for this Subnet.
    Failed,
}

#[cfg(all(feature = "cloud-engine-host", feature = "subnet-catalog-host"))]
impl CloudEngineOperatorLookupStatus {
    /// Return the stable JSON and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Absent => "absent",
            Self::Failed => "failed",
        }
    }
}

///
/// CloudEngineListRow
///
/// One Registry-classified CloudEngine Subnet and its separate operator-binding observation.
///

#[cfg(all(feature = "cloud-engine-host", feature = "subnet-catalog-host"))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CloudEngineListRow {
    /// Canonical Registry Subnet principal.
    pub subnet_id: String,
    /// Human-facing catalog label.
    pub subnet_label: String,
    /// Provenance for the catalog label.
    pub subnet_label_source: ClassificationSource,
    /// Registry Subnet type discriminant.
    pub registry_subnet_type: i32,
    /// Current catalog classification; always `cloud_engine` in this report.
    pub subnet_kind: SubnetKind,
    /// Provenance for the Subnet kind.
    pub subnet_kind_source: ClassificationSource,
    /// Current catalog specialization.
    pub subnet_specialization: SubnetSpecialization,
    /// Provenance for the specialization.
    pub subnet_specialization_source: ClassificationSource,
    /// Current catalog geographic scope.
    pub geographic_scope: GeographicScope,
    /// Provenance for the geographic scope.
    pub geographic_scope_source: ClassificationSource,
    /// Registry node count when present.
    pub node_count: Option<u32>,
    /// Whether application charges normally apply for this classification.
    pub charges_apply_by_default: bool,
    /// Number of routing ranges assigned to the Subnet in the catalog snapshot.
    pub range_count: usize,
    /// Result of the separate public control-plane lookup.
    pub operator_lookup_status: CloudEngineOperatorLookupStatus,
    /// Operator canister returned by the control plane when resolved.
    pub operator_canister_id: Option<String>,
    /// Per-row lookup failure, separate from a successful absent result.
    pub operator_lookup_error: Option<String>,
}

///
/// CloudEngineListReport
///
/// Registry CloudEngine inventory joined to bounded public operator-binding observations.
///

#[cfg(all(feature = "cloud-engine-host", feature = "subnet-catalog-host"))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CloudEngineListReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Queried network identity.
    pub network: String,

    /// Registry authority represented by the inventory side of the report.
    pub registry_authority: String,
    /// Registry canister principal represented by the catalog snapshot.
    pub registry_canister_id: String,
    /// Exact Registry version represented by the catalog snapshot.
    pub registry_version: u64,
    /// Assurance established for the Registry snapshot.
    pub registry_assurance: CatalogAssurance,
    /// Endpoints contributing to the Registry snapshot.
    pub registry_source_endpoints: Vec<String>,
    /// Endpoint-agreement digest when established by collection policy.
    pub registry_agreement_digest: Option<String>,
    /// Registry calls made when the represented catalog was collected.
    pub registry_query_call_count: u64,

    /// Local path supplying the catalog snapshot.
    pub catalog_path: String,
    /// Current catalog schema version.
    pub catalog_schema_version: u32,
    /// Canonical catalog payload digest.
    pub catalog_digest: String,
    /// Cache action supplying this report.
    pub catalog_cache_disposition: CacheDisposition,
    /// Catalog collection timestamp.
    pub catalog_fetched_at: String,
    /// Whether the catalog exceeds the display freshness threshold.
    pub catalog_stale: bool,
    /// Human-readable stale determination.
    pub catalog_stale_reason: String,
    /// Collector package version recorded by the catalog.
    pub catalog_collector_version: String,
    /// Classification contract version used by the catalog.
    pub classification_schema_version: u32,
    /// Digest of the classification policy used by the catalog.
    pub classification_policy_digest: String,
    /// Resolver backend recorded by the catalog.
    pub resolver_backend: String,
    /// Resolver contract version used by the catalog.
    pub resolver_schema_version: u32,

    /// Control-plane authority represented by binding observations.
    pub control_plane_authority: String,
    /// Fixed CloudEngine control-plane canister principal.
    pub control_plane_canister_id: String,
    /// Replica endpoint used for operator-binding lookups.
    pub control_plane_source_endpoint: String,
    /// Collection timestamp for the binding observations.
    pub control_plane_fetched_at: String,
    /// Collector identity for the binding observations.
    pub control_plane_fetched_by: String,
    /// Whether the control-plane application data was cryptographically certified.
    pub control_plane_certified: bool,
    /// Whether the per-row calls form one point-in-time view.
    pub control_plane_point_in_time_guaranteed: bool,
    /// Number of exact per-Subnet control-plane lookups attempted.
    pub control_plane_lookup_attempt_count: usize,

    /// Number of CloudEngine Subnets supplied by the Registry catalog.
    pub registry_cloud_engine_subnet_count: usize,
    /// Number of Subnets with a resolved operator binding.
    pub operator_binding_count: usize,
    /// Number of successful lookups that returned no binding.
    pub missing_operator_binding_count: usize,
    /// Number of per-row control-plane lookup failures.
    pub operator_lookup_failure_count: usize,
    /// Canonically ordered Registry inventory with separate binding results.
    pub cloud_engines: Vec<CloudEngineListRow>,
}

///
/// CloudEngineOperatorSourceData
///
/// Untrusted source result for one Subnet-to-operator lookup and public detail follow-up.
///

#[cfg(feature = "cloud-engine-host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudEngineOperatorSourceData {
    /// Source provenance echoed by the adapter.
    pub source: CloudEngineSourceRequest,
    /// Canonical Subnet principal looked up by the source.
    pub subnet_id: String,
    /// Per-engine operator canister principal when registered.
    pub operator_canister_id: Option<String>,
    /// Public engine-owner principal returned by the operator.
    pub engine_owner: Option<String>,
    /// Public platform-administrator principal returned by the operator.
    pub platform_admin: Option<String>,
    /// Public Caffeine setting returned by the operator.
    pub caffeine_enabled: Option<bool>,
    /// Claimed custom-domain names, or `None` when the field was absent.
    pub claimed_domains: Option<Vec<String>>,
    /// Exact number of native query calls made by the source.
    pub query_call_count: usize,
}

///
/// CloudEngineOperatorBindingSourceData
///
/// Untrusted source result for one exact Subnet-to-operator lookup without detail calls.
///

#[cfg(feature = "cloud-engine-host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudEngineOperatorBindingSourceData {
    /// Source provenance echoed by the adapter.
    pub source: CloudEngineSourceRequest,
    /// Canonical Subnet principal looked up by the source.
    pub subnet_id: String,
    /// Per-engine operator canister principal when registered.
    pub operator_canister_id: Option<String>,
    /// Exact number of native query calls made by the source.
    pub query_call_count: usize,
}

///
/// CloudEnginePricesSourceData
///
/// Untrusted source result for the public CloudEngine marketplace.
///

#[cfg(feature = "cloud-engine-host")]
#[derive(Clone, Debug, PartialEq)]
pub struct CloudEnginePricesSourceData {
    /// Source provenance echoed by the adapter.
    pub source: CloudEngineSourceRequest,
    /// Raw network-fee fraction returned by the control-plane canister.
    pub network_fee: f64,
    /// Raw public marketplace rows.
    pub prices: Vec<CloudEnginePriceRow>,
    /// Exact number of native query calls made by the source.
    pub query_call_count: usize,
}
