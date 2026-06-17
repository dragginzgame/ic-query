//! Module: sns::report::model::reports::proposals
//!
//! Responsibility: SNS proposal report DTOs.
//! Does not own: live governance calls, proposal conversion, or rendering.
//! Boundary: preserves proposal detail and listing fields for text and JSON.

use serde::{Deserialize as SerdeDeserialize, Serialize};

///
/// SnsProposalsCacheListReport
///
/// Serializable report listing complete local SNS proposal caches.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalsCacheListReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub cache_count: usize,
    pub caches: Vec<SnsProposalsCacheSummary>,
}

///
/// SnsProposalsCacheStatusReport
///
/// Serializable report describing one expected or discovered SNS proposal cache.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalsCacheStatusReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub input: String,
    pub found: bool,
    pub cache: Option<SnsProposalsCacheSummary>,
    pub expected_cache_path: Option<String>,
    pub refresh_attempt_path: Option<String>,
    pub latest_attempt: Option<SnsProposalsRefreshAttemptStatus>,
}

///
/// SnsProposalsCacheSummary
///
/// Serializable summary of one complete SNS proposal snapshot cache.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalsCacheSummary {
    pub id: usize,
    pub name: String,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub complete: bool,
    pub row_count: usize,
    pub page_count: u32,
    pub page_size: u32,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub cache_path: String,
    pub refresh_attempt_path: String,
    pub latest_attempt: Option<SnsProposalsRefreshAttemptStatus>,
}

///
/// SnsProposalsRefreshAttemptStatus
///
/// Serializable status for the latest SNS proposal snapshot refresh attempt.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalsRefreshAttemptStatus {
    pub status: String,
    pub started_at: String,
    pub updated_at: String,
    pub page_size: u32,
    pub pages_fetched: u32,
    pub rows_fetched: usize,
    pub last_cursor: Option<String>,
    pub last_error: Option<String>,
}

///
/// SnsProposalsRefreshReport
///
/// Serializable report returned after a complete SNS proposal snapshot refresh.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalsRefreshReport {
    pub schema_version: u32,
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub id: usize,
    pub name: String,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub cache_path: String,
    pub refresh_lock_path: String,
    pub refresh_attempt_path: String,
    pub page_size: u32,
    pub page_count: u32,
    pub proposal_count: usize,
    pub complete: bool,
    pub replaced_existing_cache: bool,
    pub wrote_cache: bool,
}

///
/// SnsProposalReport
///
/// Serializable report for one SNS governance proposal detail lookup.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalReport {
    pub schema_version: u32,
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub id: usize,
    pub name: String,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub proposal_id: u64,
    pub verbose: bool,
    pub show_ballots: bool,
    pub proposal: SnsProposalRow,
}

///
/// SnsProposalsReport
///
/// Serializable report for a bounded SNS governance proposal listing.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalsReport {
    pub schema_version: u32,
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub id: usize,
    pub name: String,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub requested_limit: u32,
    pub before_proposal_id: Option<u64>,
    pub status_filter: String,
    pub topic_filter: String,
    pub verbose: bool,
    pub proposal_count: usize,
    pub proposals: Vec<SnsProposalRow>,
}

///
/// SnsProposalRow
///
/// Serializable row for one SNS governance proposal.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct SnsProposalRow {
    pub proposal_id: Option<u64>,
    pub action_id: u64,
    pub action: String,
    pub title: String,
    pub summary: String,
    pub url: Option<String>,
    pub decision_state: String,
    pub reject_cost_e8s: u64,
    pub proposal_creation_timestamp_seconds: u64,
    pub created_at: String,
    pub decided_timestamp_seconds: Option<u64>,
    pub decided_at: Option<String>,
    pub executed_timestamp_seconds: Option<u64>,
    pub executed_at: Option<String>,
    pub failed_timestamp_seconds: Option<u64>,
    pub failed_at: Option<String>,
    pub failure_reason: Option<SnsProposalFailureReason>,
    pub reward_event_round: u64,
    pub reward_event_end_timestamp_seconds: Option<u64>,
    pub is_eligible_for_rewards: bool,
    pub latest_tally: Option<SnsProposalTally>,
    pub ballot_count: usize,
    pub ballots: Vec<SnsProposalBallotRow>,
    pub payload_text_rendering: Option<String>,
    pub proposer_neuron_id: Option<String>,
}

///
/// SnsProposalBallotRow
///
/// Serializable row for one proposal ballot.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct SnsProposalBallotRow {
    pub neuron_id: String,
    pub vote: i32,
    pub vote_text: String,
    pub cast_timestamp_seconds: u64,
    pub cast_at: Option<String>,
    pub voting_power: u64,
}

///
/// SnsProposalFailureReason
///
/// Serializable SNS governance failure reason attached to a proposal.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct SnsProposalFailureReason {
    pub error_type: i32,
    pub error_message: String,
}

///
/// SnsProposalTally
///
/// Serializable SNS proposal vote tally.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct SnsProposalTally {
    pub timestamp_seconds: u64,
    pub yes: u64,
    pub no: u64,
    pub total: u64,
}
