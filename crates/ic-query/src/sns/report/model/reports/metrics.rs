//! Module: sns::report::model::reports::metrics
//!
//! Responsibility: bounded SNS Governance metrics report DTOs.
//! Does not own: discovery, live calls, source validation, or rendering.
//! Boundary: preserves raw cached treasury evidence and native optional values.

use super::invocation::{SnsCanisterCallType, SnsCanisterMethod};
use serde::Serialize;

///
/// MainnetSnsMetrics
///
/// Source-layer result from one bounded SNS Governance metrics query.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MainnetSnsMetrics {
    /// Governance canister identity queried by the source.
    pub governance_canister_id: String,
    /// Native Governance method queried by the source.
    pub method: SnsCanisterMethod,
    /// Native call type used by the source.
    pub call_type: SnsCanisterCallType,
    /// Proposal-count window supplied to Governance.
    pub time_window_seconds: u64,
    /// Whether the source can prove one point-in-time snapshot.
    pub point_in_time_guaranteed: bool,
    /// Whether treasury values are cached Governance metrics.
    pub treasury_metrics_cached: bool,
    /// Recent submitted-proposal count.
    pub num_recently_submitted_proposals: Option<u64>,
    /// Recent executed-proposal count.
    pub num_recently_executed_proposals: Option<u64>,
    /// Latest SNS-ledger block timestamp observed by Governance.
    pub last_ledger_block_timestamp: Option<u64>,
    /// SNS genesis timestamp.
    pub genesis_timestamp_seconds: Option<u64>,
    /// Cached treasury metrics.
    pub treasury_metrics: Vec<SnsTreasuryMetricRow>,
    /// Cached voting-power metrics.
    pub voting_power_metrics: Option<SnsVotingPowerMetrics>,
}

///
/// SnsTreasuryMetricRow
///
/// One timestamped cached treasury metric returned by SNS Governance.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsTreasuryMetricRow {
    /// Raw native treasury discriminant.
    pub treasury: i32,
    /// Human-readable treasury name returned by Governance.
    pub name: Option<String>,
    /// Ledger that is authoritative for the treasury account.
    pub ledger_canister_id: Option<String>,
    /// Owner of the authoritative treasury account.
    pub account_owner: Option<String>,
    /// Optional 32-byte account subaccount as lowercase hexadecimal text.
    pub account_subaccount_hex: Option<String>,
    /// Cached current treasury amount in native e8s.
    pub amount_e8s: Option<u64>,
    /// Treasury amount at swap finalization in native e8s.
    pub original_amount_e8s: Option<u64>,
    /// Unix timestamp when this cached treasury metric was updated.
    pub timestamp_seconds: Option<u64>,
}

///
/// SnsVotingPowerMetrics
///
/// Timestamped cached SNS Governance voting-power metrics.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsVotingPowerMetrics {
    /// Total potential Governance voting power, when returned.
    pub governance_total_potential_voting_power: Option<u64>,
    /// Unix timestamp when the voting-power metric was updated.
    pub timestamp_seconds: Option<u64>,
}

///
/// SnsMetricsReport
///
/// Bounded live report of native SNS Governance metrics for one SNS.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsMetricsReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Requested IC network identity.
    pub network: String,
    /// Mainnet SNS-W canister used for discovery.
    pub sns_wasm_canister_id: String,
    /// Collection timestamp in UTC.
    pub fetched_at: String,
    /// IC API endpoint used for all client requests.
    pub source_endpoint: String,
    /// Collector identity recorded by the source request.
    pub fetched_by: String,
    /// SNS-W list id assigned to this deployed SNS.
    pub id: usize,
    /// SNS name resolved during discovery.
    pub name: String,
    /// Root canister identity used to resolve this SNS.
    pub root_canister_id: String,
    /// Number of returned cached treasury rows.
    pub treasury_metric_count: usize,

    /// Validated source payload shared with native collection.
    #[serde(flatten)]
    pub metrics: MainnetSnsMetrics,
}

pub(in crate::sns::report) const fn sns_treasury_kind(treasury: i32) -> &'static str {
    match treasury {
        0 => "unspecified",
        1 => "icp",
        2 => "sns_token",
        _ => "unknown",
    }
}
