use candid::{CandidType, Deserialize};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{io, path::PathBuf};
use thiserror::Error as ThisError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsListRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub verbose: bool,
    pub sort: SnsListSort,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsLookupRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub input: String,
}

pub type SnsInfoRequest = SnsLookupRequest;
pub type SnsParamsRequest = SnsLookupRequest;
pub type SnsTokenRequest = SnsLookupRequest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsNeuronsRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub input: String,
    pub limit: u32,
    pub owner_principal_id: Option<String>,
    pub sort: SnsNeuronsSort,
    pub icp_root: Option<PathBuf>,
    pub verbose: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnsNeuronsRefreshRequest {
    pub network: String,
    pub source_endpoint: String,
    pub now_unix_secs: u64,
    pub input: String,
    pub icp_root: PathBuf,
    pub page_size: u32,
    pub max_pages: Option<u32>,
}

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

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct SnsNeuronRow {
    pub neuron_id: String,
    pub cached_neuron_stake_e8s: u64,
    pub maturity_e8s_equivalent: u64,
    pub staked_maturity_e8s_equivalent: Option<u64>,
    pub created_timestamp_seconds: u64,
    pub created_at: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SnsNeuronsSort {
    #[default]
    Api,
    Id,
    Stake,
    Maturity,
    Created,
}

impl SnsNeuronsSort {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::Id => "id",
            Self::Stake => "stake",
            Self::Maturity => "maturity",
            Self::Created => "created",
        }
    }

    #[must_use]
    pub const fn uses_cache(self) -> bool {
        !matches!(self, Self::Api)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SnsListSort {
    #[default]
    Id,
    Name,
}

impl SnsListSort {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Id => "id",
            Self::Name => "name",
        }
    }
}

#[derive(Debug, ThisError)]
pub enum SnsHostError {
    #[error(
        "`icq sns` supports only the mainnet `ic` network\n\nThe SNS list is queried from the public Internet Computer mainnet SNS-W canister.\nLocal replica SNS discovery is not implemented yet.\n\nTry:\n  icq --network ic sns list"
    )]
    UnsupportedNetwork { network: String },

    #[error("failed to create Tokio runtime for SNS query: {0}")]
    Runtime(String),

    #[error("failed to build IC agent for endpoint {endpoint}: {reason}")]
    AgentBuild { endpoint: String, reason: String },

    #[error("invalid {field}: {reason}")]
    InvalidPrincipal { field: &'static str, reason: String },

    #[error("failed to encode Candid request for {message}: {reason}")]
    CandidEncode {
        message: &'static str,
        reason: String,
    },

    #[error("SNS query method {method} failed: {reason}")]
    AgentCall {
        method: &'static str,
        reason: String,
    },

    #[error("failed to decode Candid response {message}: {reason}")]
    CandidDecode {
        message: &'static str,
        reason: String,
    },

    #[error("SNS list id {id} is out of range; list contains {sns_count} deployed SNS instances")]
    UnknownSnsId { id: usize, sns_count: usize },

    #[error("could not find deployed SNS with root principal {root_canister_id}")]
    UnknownSnsRoot { root_canister_id: String },

    #[error("SNS lookup input must be a list id or root principal: {input}")]
    InvalidLookup { input: String },

    #[error(
        "SNS neurons cache is missing at {}\n\nRun `icq sns neurons refresh <id|root-principal>` to fetch a complete snapshot before using cache-backed sorting.",
        path.display()
    )]
    MissingNeuronsCache { path: PathBuf },

    #[error("failed to read SNS cache at {}: {source}", path.display())]
    ReadCache { path: PathBuf, source: io::Error },

    #[error("failed to parse SNS cache at {}: {source}", path.display())]
    ParseCache {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("failed to serialize SNS cache JSON for {}: {source}", path.display())]
    SerializeCache {
        path: PathBuf,
        source: serde_json::Error,
    },

    #[error("unsupported SNS cache schema version {version}; expected {expected}")]
    UnsupportedCacheSchemaVersion { version: u32, expected: u32 },

    #[error("cached SNS network mismatch: path is for {requested}, report is for {actual}")]
    CacheNetworkMismatch { requested: String, actual: String },

    #[error("SNS cache operation failed: {0}")]
    Cache(String),

    #[error(
        "SNS neurons refresh did not publish a complete snapshot after {pages_fetched} pages and {rows_fetched} rows: {reason}"
    )]
    IncompleteRefresh {
        pages_fetched: u32,
        rows_fetched: usize,
        reason: String,
    },

    #[error("SNS cache root is required for cache-backed neuron reports")]
    MissingCacheRoot,
}
