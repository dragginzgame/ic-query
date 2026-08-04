//! Module: sns::report::model::reports::metrics
//!
//! Responsibility: bounded SNS Governance metrics report DTOs.
//! Does not own: discovery, live calls, source validation, or rendering.
//! Boundary: preserves raw cached treasury evidence and native optional values.

use super::invocation::{SnsCanisterCallType, SnsCanisterMethod};
use serde::Serialize;

///
/// SnsTreasuryKind
///
/// Native SNS treasury asset classification with unknown-value preservation.
///

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnsTreasuryKind {
    /// Native code zero.
    Unspecified,
    /// Native ICP treasury code.
    Icp,
    /// Native SNS governance-token treasury code.
    SnsToken,
    /// Future or otherwise unknown native code.
    Unknown,
}

impl SnsTreasuryKind {
    /// Return the stable native classification label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unspecified => "unspecified",
            Self::Icp => "icp",
            Self::SnsToken => "sns_token",
            Self::Unknown => "unknown",
        }
    }
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
    /// Native treasury classification derived from the raw discriminant.
    pub treasury_kind: SnsTreasuryKind,
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
    /// Governance canister queried for metrics.
    pub governance_canister_id: String,
    /// Native Governance metrics method.
    pub method: SnsCanisterMethod,
    /// Native call type used for the Governance method.
    pub call_type: SnsCanisterCallType,
    /// Requested recent-proposal window in seconds.
    pub time_window_seconds: u64,
    /// Whether all returned metrics represent one authoritative point in time.
    pub point_in_time_guaranteed: bool,
    /// Whether treasury values are cached Governance metrics.
    pub treasury_metrics_cached: bool,
    /// Recent submitted-proposal count for the requested window.
    pub num_recently_submitted_proposals: Option<u64>,
    /// Recent executed-proposal count for the requested window.
    pub num_recently_executed_proposals: Option<u64>,
    /// Latest SNS-ledger block timestamp observed by Governance.
    pub last_ledger_block_timestamp: Option<u64>,
    /// SNS genesis timestamp returned by Governance.
    pub genesis_timestamp_seconds: Option<u64>,
    /// Number of returned cached treasury rows.
    pub treasury_metric_count: usize,
    /// Canonically ordered cached treasury metrics.
    pub treasury_metrics: Vec<SnsTreasuryMetricRow>,
    /// Cached voting-power metrics, when returned.
    pub voting_power_metrics: Option<SnsVotingPowerMetrics>,
}
