//! Module: sns::report::model::reports::list
//!
//! Responsibility: deployed SNS list and info report DTOs.
//! Does not own: SNS-W fetching, metadata lookup, sorting, or rendering.
//! Boundary: preserves raw report fields for text and JSON output writers.

use crate::report::ReportProvenance;
use crate::sns::report::model::MainnetSnsCanisters;
use serde::Serialize;

///
/// SnsListReport
///
/// Serializable report for deployed SNS listings.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsListReport {
    pub schema_version: u32,
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    /// Shared record, flattened in serialized output.
    #[serde(flatten)]
    pub provenance: ReportProvenance,
    /// Whether the view includes SNS instances outside the normal visible lifecycle set.
    pub all_lifecycles: bool,
    pub sort: String,
    /// Number of SNS instances in the complete joined catalog before view filtering.
    pub catalog_sns_count: usize,
    /// Number of catalog rows excluded by lifecycle filtering.
    pub excluded_sns_count: usize,
    pub sns_count: usize,
    pub metadata_error_count: usize,
    /// Number of returned rows carrying a bounded lifecycle query error.
    pub lifecycle_error_count: usize,
    pub sns_instances: Vec<SnsListRow>,
}

///
/// SnsLifecycleOutcome
///
/// Raw Swap lifecycle evidence, including queries not requested for direct lookup.
///

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, Serialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum SnsLifecycleOutcome {
    /// This lookup did not request lifecycle evidence.
    NotCollected,
    /// Successful query, preserving every raw discriminant.
    Value {
        /// Raw successful Swap discriminant.
        code: i32,
    },
    /// Bounded query failure retained with the SNS identity.
    Failed {
        /// Bounded original query failure.
        error: String,
    },
}

impl SnsLifecycleOutcome {
    /// Return the raw successful lifecycle code.
    #[must_use]
    pub const fn value(&self) -> Option<i32> {
        match self {
            Self::Value { code } => Some(*code),
            Self::NotCollected | Self::Failed { .. } => None,
        }
    }

    /// Return retained failure evidence.
    #[must_use]
    pub fn error(&self) -> Option<&str> {
        match self {
            Self::Failed { error } => Some(error),
            Self::NotCollected | Self::Value { .. } => None,
        }
    }
}

///
/// SnsListRow
///
/// Serializable row for one deployed SNS in a list report.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsListRow {
    pub id: usize,
    pub name: String,
    /// Complete deployed-SNS canister identity, serialized as the existing flat fields.
    #[serde(flatten)]
    pub canisters: MainnetSnsCanisters,
    pub metadata_error: Option<String>,
    /// Collected lifecycle value or bounded query failure.
    pub lifecycle: SnsLifecycleOutcome,
}

///
/// SnsInfoReport
///
/// Serializable report for one deployed SNS resolved by id or root principal.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsInfoReport {
    pub schema_version: u32,
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub id: usize,
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    /// Complete deployed-SNS canister identity, serialized as the existing flat fields.
    #[serde(flatten)]
    pub canisters: MainnetSnsCanisters,
    pub metadata_error: Option<String>,
}
