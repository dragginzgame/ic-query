use candid::{CandidType, Deserialize};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsListReport {
    pub schema_version: u32,
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub verbose: bool,
    pub sort: String,
    pub sns_count: usize,
    pub metadata_error_count: usize,
    pub sns_instances: Vec<SnsListRow>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsListRow {
    pub id: usize,
    pub name: String,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub ledger_canister_id: String,
    pub swap_canister_id: String,
    pub index_canister_id: String,
    pub metadata_error: Option<String>,
}

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
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub ledger_canister_id: String,
    pub swap_canister_id: String,
    pub index_canister_id: String,
    pub metadata_error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsTokenReport {
    pub schema_version: u32,
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub id: usize,
    pub name: String,
    pub root_canister_id: String,
    pub ledger_canister_id: String,
    pub sns_index_canister_id: String,
    pub token_name: String,
    pub token_symbol: String,
    pub decimals: u8,
    pub transfer_fee: String,
    pub total_supply: String,
    pub minting_account_owner: Option<String>,
    pub minting_account_subaccount_hex: Option<String>,
    pub ledger_index_canister_id: Option<String>,
    pub ledger_index_error: Option<String>,
    pub supported_standards: Vec<SnsTokenStandardRow>,
    pub metadata: Vec<SnsTokenMetadataRow>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsTokenStandardRow {
    pub name: String,
    pub url: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsTokenMetadataRow {
    pub key: String,
    pub value_type: String,
    pub value: JsonValue,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsParamsReport {
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
    pub parameters: SnsGovernanceParameters,
}

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

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize, Serialize)]
pub struct SnsGovernanceParameters {
    pub max_dissolve_delay_seconds: Option<u64>,
    pub max_dissolve_delay_bonus_percentage: Option<u64>,
    pub max_followees_per_function: Option<u64>,
    pub neuron_claimer_permissions: Option<SnsNeuronPermissionList>,
    pub neuron_minimum_stake_e8s: Option<u64>,
    pub max_neuron_age_for_age_bonus: Option<u64>,
    pub initial_voting_period_seconds: Option<u64>,
    pub neuron_minimum_dissolve_delay_to_vote_seconds: Option<u64>,
    pub reject_cost_e8s: Option<u64>,
    pub max_proposals_to_keep_per_action: Option<u32>,
    pub wait_for_quiet_deadline_increase_seconds: Option<u64>,
    pub max_number_of_neurons: Option<u64>,
    pub transaction_fee_e8s: Option<u64>,
    pub max_number_of_proposals_with_ballots: Option<u64>,
    pub max_age_bonus_percentage: Option<u64>,
    pub neuron_grantable_permissions: Option<SnsNeuronPermissionList>,
    pub voting_rewards_parameters: Option<SnsVotingRewardsParameters>,
    pub maturity_modulation_disabled: Option<bool>,
    pub max_number_of_principals_per_neuron: Option<u64>,
    pub automatically_advance_target_version: Option<bool>,
    pub custom_proposal_criticality: Option<SnsCustomProposalCriticality>,
}

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize, Serialize)]
pub struct SnsNeuronPermissionList {
    pub permissions: Vec<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize, Serialize)]
pub struct SnsVotingRewardsParameters {
    pub final_reward_rate_basis_points: Option<u64>,
    pub initial_reward_rate_basis_points: Option<u64>,
    pub reward_rate_transition_duration_seconds: Option<u64>,
    pub round_duration_seconds: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize, Serialize)]
pub struct SnsCustomProposalCriticality {
    pub additional_critical_native_action_ids: Vec<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsNeuronsReport {
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
    pub owner_principal_id: Option<String>,
    pub verbose: bool,
    pub data_source: String,
    pub sort: String,
    pub cache_path: Option<String>,
    pub cache_complete: Option<bool>,
    pub total_neuron_count: usize,
    pub neuron_count: usize,
    pub neurons: Vec<SnsNeuronRow>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsNeuronsRefreshReport {
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
    pub neuron_count: usize,
    pub complete: bool,
    pub replaced_existing_cache: bool,
    pub wrote_cache: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsNeuronsCacheListReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub cache_count: usize,
    pub caches: Vec<SnsNeuronsCacheSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsNeuronsCacheStatusReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub input: String,
    pub found: bool,
    pub cache: Option<SnsNeuronsCacheSummary>,
    pub expected_cache_path: Option<String>,
    pub refresh_attempt_path: Option<String>,
    pub latest_attempt: Option<SnsNeuronsRefreshAttemptStatus>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsNeuronsCacheSummary {
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
    pub latest_attempt: Option<SnsNeuronsRefreshAttemptStatus>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsNeuronsRefreshAttemptStatus {
    pub status: String,
    pub started_at: String,
    pub updated_at: String,
    pub page_size: u32,
    pub pages_fetched: u32,
    pub rows_fetched: usize,
    pub last_cursor: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct SnsNeuronRow {
    pub neuron_id: String,
    pub cached_neuron_stake_e8s: u64,
    pub maturity_e8s_equivalent: u64,
    pub staked_maturity_e8s_equivalent: Option<u64>,
    pub created_timestamp_seconds: u64,
    pub created_at: String,
}
