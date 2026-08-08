//! Module: cloud_engine::model
//!
//! Responsibility: define stable CloudEngine report and source-data models.
//! Does not own: live transport, source validation, CLI parsing, or text rendering.
//! Boundary: preserves raw control-plane identities, prices, timestamps, and authority limits.

use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(feature = "cloud-engine-host")]
use super::CloudEngineSourceRequest;

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
