//! Module: sns::report::live::types::metrics
//!
//! Responsibility: SNS Governance metrics Candid wire types.
//! Does not own: transport, source validation, report assembly, or rendering.
//! Boundary: mirrors only the official bounded metrics response fields.

use super::SnsGovernanceError;
use candid::{CandidType, Deserialize, Principal};

///
/// GetMetricsRequest
///
/// Native SNS Governance metrics request.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetMetricsRequest {
    pub(in crate::sns::report::live) time_window_seconds: Option<u64>,
}

///
/// SnsMetricsSubaccount
///
/// Native optional account subaccount wrapper.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsMetricsSubaccount {
    pub(in crate::sns::report::live) subaccount: Vec<u8>,
}

///
/// SnsMetricsAccount
///
/// Native treasury account returned by Governance.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsMetricsAccount {
    pub(in crate::sns::report::live) owner: Option<Principal>,
    pub(in crate::sns::report::live) subaccount: Option<SnsMetricsSubaccount>,
}

///
/// TreasuryMetricsWire
///
/// Native cached treasury metric returned by Governance.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct TreasuryMetricsWire {
    pub(in crate::sns::report::live) treasury: i32,
    pub(in crate::sns::report::live) name: Option<String>,
    pub(in crate::sns::report::live) ledger_canister_id: Option<Principal>,
    pub(in crate::sns::report::live) account: Option<SnsMetricsAccount>,
    pub(in crate::sns::report::live) amount_e8s: Option<u64>,
    pub(in crate::sns::report::live) original_amount_e8s: Option<u64>,
    pub(in crate::sns::report::live) timestamp_seconds: Option<u64>,
}

///
/// VotingPowerMetricsWire
///
/// Native cached voting-power metric returned by Governance.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct VotingPowerMetricsWire {
    pub(in crate::sns::report::live) governance_total_potential_voting_power: Option<u64>,
    pub(in crate::sns::report::live) timestamp_seconds: Option<u64>,
}

///
/// MetricsWire
///
/// Native successful SNS Governance metrics payload.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct MetricsWire {
    pub(in crate::sns::report::live) num_recently_submitted_proposals: Option<u64>,
    pub(in crate::sns::report::live) num_recently_executed_proposals: Option<u64>,
    pub(in crate::sns::report::live) last_ledger_block_timestamp: Option<u64>,
    pub(in crate::sns::report::live) treasury_metrics: Option<Vec<TreasuryMetricsWire>>,
    pub(in crate::sns::report::live) voting_power_metrics: Option<VotingPowerMetricsWire>,
    pub(in crate::sns::report::live) genesis_timestamp_seconds: Option<u64>,
}

///
/// GetMetricsResult
///
/// Native SNS Governance metrics result variant.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) enum GetMetricsResult {
    Ok(MetricsWire),
    Err(SnsGovernanceError),
}

///
/// GetMetricsResponse
///
/// Native SNS Governance metrics response.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetMetricsResponse {
    pub(in crate::sns::report::live) get_metrics_result: Option<GetMetricsResult>,
}
