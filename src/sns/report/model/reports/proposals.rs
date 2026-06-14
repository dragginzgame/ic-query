use serde::Serialize;

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
    pub proposal: SnsProposalRow,
}

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
    pub verbose: bool,
    pub proposal_count: usize,
    pub proposals: Vec<SnsProposalRow>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalBallotRow {
    pub neuron_id: String,
    pub vote: i32,
    pub vote_text: String,
    pub cast_timestamp_seconds: u64,
    pub cast_at: Option<String>,
    pub voting_power: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalFailureReason {
    pub error_type: i32,
    pub error_message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsProposalTally {
    pub timestamp_seconds: u64,
    pub yes: u64,
    pub no: u64,
    pub total: u64,
}
