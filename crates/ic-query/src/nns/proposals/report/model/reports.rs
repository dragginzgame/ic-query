//! Module: nns::proposals::report::model::reports
//!
//! Responsibility: serialized NNS proposal report and row DTOs.
//! Does not own: request selection, source transport, or text rendering.
//! Boundary: defines the stable JSON contract for NNS proposal output.

use super::selection::{
    NnsProposalRewardStatus, NnsProposalStatus, NnsProposalTopic, NnsProposalVote,
};
use serde::{Deserialize, Serialize};

///
/// NnsProposalListReport
///
/// Serializable report for a bounded NNS governance proposal listing.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsProposalListReport {
    pub schema_version: u32,
    pub network: String,
    pub governance_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub data_source: String,
    pub cache_path: Option<String>,
    pub cache_complete: Option<bool>,
    pub requested_limit: u32,
    pub before_proposal_id: Option<u64>,
    pub status_filter: String,
    pub reward_status_filter: String,
    pub topic_filter: String,
    pub proposer_filter: Option<u64>,
    pub query_filter: Option<String>,
    pub sort: String,
    pub sort_direction: String,
    pub result_scope: String,
    pub verbose: bool,
    pub proposal_count: usize,
    pub proposals: Vec<NnsProposalRow>,
}

///
/// NnsProposalReport
///
/// Serializable report for one NNS governance proposal detail lookup.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsProposalReport {
    pub schema_version: u32,
    pub network: String,
    pub governance_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub data_source: String,
    pub cache_path: Option<String>,
    pub cache_complete: Option<bool>,
    pub proposal_id: u64,
    pub show_ballots: bool,
    pub verbose: bool,
    pub proposal: NnsProposalRow,
}

///
/// NnsProposalRow
///
/// Serializable row for one NNS governance proposal.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsProposalRow {
    pub proposal_id: Option<u64>,
    pub proposer_neuron_id: Option<u64>,
    pub topic: i32,
    pub topic_text: NnsProposalTopic,
    pub status: i32,
    pub status_text: NnsProposalStatus,
    pub reward_status: i32,
    pub reward_status_text: NnsProposalRewardStatus,
    pub title: Option<String>,
    pub summary: String,
    pub url: String,
    pub action_text: Option<String>,
    pub reject_cost_e8s: u64,
    pub proposal_timestamp_seconds: u64,
    pub proposed_at: String,
    pub deadline_timestamp_seconds: Option<u64>,
    pub deadline_at: Option<String>,
    pub decided_timestamp_seconds: u64,
    pub decided_at: Option<String>,
    pub executed_timestamp_seconds: u64,
    pub executed_at: Option<String>,
    pub failed_timestamp_seconds: u64,
    pub failed_at: Option<String>,
    pub reward_event_round: u64,
    pub total_potential_voting_power: Option<u64>,
    pub latest_tally: Option<NnsProposalTally>,
    pub ballot_count: usize,
    pub ballots: Vec<NnsProposalBallotRow>,
}

///
/// NnsProposalBallotRow
///
/// Serializable NNS proposal ballot row.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsProposalBallotRow {
    pub neuron_id: u64,
    pub vote: i32,
    pub vote_text: NnsProposalVote,
    pub voting_power: u64,
}

///
/// NnsProposalTally
///
/// Serializable NNS proposal vote tally.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsProposalTally {
    pub timestamp_seconds: u64,
    pub yes: u64,
    pub no: u64,
    pub total: u64,
}
