use crate::{
    cache_file::{
        CacheFileError, CachedJsonReport, JsonCacheReport, LoadJsonCacheErrorHandlers,
        LoadJsonCacheRequest, RefreshLockRequest, create_directory, load_json_cache,
        with_refresh_lock, write_text_atomically,
    },
    ic_registry::DEFAULT_MAINNET_ENDPOINT,
    nns::render::yes_no,
    progress::ProgressLine,
    runtime::block_on_current_thread,
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
    table::{ColumnAlign, render_table},
    token_amount::{base_units_decimal_text, e8s_decimal_text},
};
use candid::{CandidType, Decode, Deserialize, Encode, Int, Nat, Principal};
use futures::{StreamExt, stream};
use ic_agent::Agent;
use serde::{Deserialize as SerdeDeserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{
    cmp::Reverse,
    collections::HashSet,
    io,
    path::{Path, PathBuf},
};
use thiserror::Error as ThisError;

pub const DEFAULT_SNS_SOURCE_ENDPOINT: &str = DEFAULT_MAINNET_ENDPOINT;
pub const MAINNET_SNS_WASM_CANISTER_ID: &str = "qaa6y-5yaaa-aaaaa-aaafa-cai";

const SNS_LIST_REPORT_SCHEMA_VERSION: u32 = 3;
const SNS_INFO_REPORT_SCHEMA_VERSION: u32 = 2;
const SNS_TOKEN_REPORT_SCHEMA_VERSION: u32 = 1;
const SNS_PARAMS_REPORT_SCHEMA_VERSION: u32 = 1;
const SNS_NEURONS_REPORT_SCHEMA_VERSION: u32 = 1;
const SNS_NEURONS_CACHE_SCHEMA_VERSION: u32 = 1;
const SNS_NEURONS_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
const SNS_NEURONS_REFRESH_ATTEMPT_SCHEMA_VERSION: u32 = 1;
const SNS_NEURONS_REFRESH_LOCK_STALE_AFTER_SECONDS: u64 = 30 * 60;
const COMPACT_PRINCIPAL_CHARS: usize = 5;
const COMPACT_NEURON_ID_CHARS: usize = 8;
const SNS_TOKEN_LOGO_METADATA_KEY: &str = "icrc1:logo";
const SNS_TOKEN_METADATA_TEXT_VALUE_LIMIT: usize = 160;
const SNS_METADATA_CONCURRENCY: usize = 16;

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

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
struct SnsNeuronsCache {
    schema_version: u32,
    network: String,
    sns_wasm_canister_id: String,
    fetched_at: String,
    source_endpoint: String,
    fetched_by: String,
    id: usize,
    name: String,
    root_canister_id: String,
    governance_canister_id: String,
    completeness: SnsNeuronsCompleteness,
    neurons: Vec<SnsNeuronRow>,
}

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
struct SnsNeuronsCompleteness {
    status: String,
    page_size: u32,
    page_count: u32,
    row_count: usize,
    point_in_time_guaranteed: bool,
}

impl JsonCacheReport for SnsNeuronsCache {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn network(&self) -> &str {
        &self.network
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct SnsNeuronsRefreshAttempt {
    schema_version: u32,
    network: String,
    source_endpoint: String,
    started_at: String,
    updated_at: String,
    root_canister_id: String,
    governance_canister_id: String,
    status: String,
    page_size: u32,
    pages_fetched: u32,
    rows_fetched: usize,
    last_cursor: Option<String>,
    last_error: Option<String>,
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

pub fn build_sns_list_report(request: &SnsListRequest) -> Result<SnsListReport, SnsHostError> {
    build_sns_list_report_with_source(request, &LiveSnsListSource)
}

pub fn build_sns_info_report(request: &SnsInfoRequest) -> Result<SnsInfoReport, SnsHostError> {
    build_sns_info_report_with_source(request, &LiveSnsListSource)
}

pub fn build_sns_params_report(
    request: &SnsParamsRequest,
) -> Result<SnsParamsReport, SnsHostError> {
    build_sns_params_report_with_source(request, &LiveSnsListSource)
}

pub fn build_sns_token_report(request: &SnsTokenRequest) -> Result<SnsTokenReport, SnsHostError> {
    build_sns_token_report_with_source(request, &LiveSnsListSource)
}

pub fn build_sns_neurons_report(
    request: &SnsNeuronsRequest,
) -> Result<SnsNeuronsReport, SnsHostError> {
    build_sns_neurons_report_with_source(request, &LiveSnsListSource)
}

pub fn refresh_sns_neurons_cache(
    request: &SnsNeuronsRefreshRequest,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    refresh_sns_neurons_cache_with_source(request, &LiveSnsListSource)
}

fn build_sns_list_report_with_source(
    request: &SnsListRequest,
    source: &dyn SnsListSource,
) -> Result<SnsListReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let fetch_request = fetch_request_from_parts(
        &request.source_endpoint,
        request.now_unix_secs,
        "ic-query".to_string(),
    );
    let mut list = source.fetch_deployed_snses(&fetch_request)?;
    assign_sns_ids_in_current_order(&mut list.sns_instances);
    sort_mainnet_sns_instances(&mut list.sns_instances, request.sort);
    Ok(sns_list_report_from_list(
        list,
        request.verbose,
        request.sort,
    ))
}

fn build_sns_info_report_with_source(
    request: &SnsInfoRequest,
    source: &dyn SnsListSource,
) -> Result<SnsInfoReport, SnsHostError> {
    let (_fetch_request, list, id, sns) = resolve_sns_lookup(request, source)?;
    Ok(sns_info_report_from_list(list, id, sns))
}

fn build_sns_params_report_with_source(
    request: &SnsParamsRequest,
    source: &dyn SnsParamsSource,
) -> Result<SnsParamsReport, SnsHostError> {
    let (fetch_request, list, id, sns) = resolve_sns_lookup(request, source)?;
    let parameters = source.fetch_sns_params(&fetch_request, &sns)?;
    Ok(sns_params_report_from_parts(list, id, sns, parameters))
}

fn build_sns_token_report_with_source(
    request: &SnsTokenRequest,
    source: &dyn SnsTokenSource,
) -> Result<SnsTokenReport, SnsHostError> {
    let (fetch_request, list, id, sns) = resolve_sns_lookup(request, source)?;
    let token = source.fetch_sns_token(&fetch_request, &sns)?;
    Ok(sns_token_report_from_parts(list, id, sns, token))
}

fn build_sns_neurons_report_with_source(
    request: &SnsNeuronsRequest,
    source: &dyn SnsNeuronsSource,
) -> Result<SnsNeuronsReport, SnsHostError> {
    let lookup_request = SnsLookupRequest {
        network: request.network.clone(),
        source_endpoint: request.source_endpoint.clone(),
        now_unix_secs: request.now_unix_secs,
        input: request.input.clone(),
    };
    let (fetch_request, list, id, sns) = resolve_sns_lookup(&lookup_request, source)?;
    if request.sort.uses_cache() {
        let icp_root = request
            .icp_root
            .as_ref()
            .ok_or(SnsHostError::MissingCacheRoot)?;
        let mut cache = load_sns_neurons_cache(icp_root, &request.network, &sns.root_canister_id)?;
        sort_sns_neurons(&mut cache.neurons, request.sort);
        let total_neuron_count = cache.neurons.len();
        let limit = usize::try_from(request.limit).unwrap_or(usize::MAX);
        cache.neurons.truncate(limit);
        let cache_path = sns_neurons_cache_path(icp_root, &request.network, &sns.root_canister_id);
        return Ok(sns_neurons_report_from_cache(SnsNeuronsCachedReportParts {
            list,
            id,
            sns,
            requested_limit: request.limit,
            sort: request.sort,
            cache,
            total_neuron_count,
            cache_path,
            verbose: request.verbose,
        }));
    }
    let neurons = source.fetch_sns_neurons(
        &fetch_request,
        &sns,
        request.limit,
        request.owner_principal_id.as_deref(),
    )?;
    Ok(sns_neurons_report_from_parts(SnsNeuronsLiveReportParts {
        list,
        id,
        sns,
        requested_limit: request.limit,
        owner_principal_id: request.owner_principal_id.clone(),
        sort: request.sort,
        verbose: request.verbose,
        neurons,
    }))
}

fn refresh_sns_neurons_cache_with_source(
    request: &SnsNeuronsRefreshRequest,
    source: &dyn SnsNeuronsSource,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let lookup_request = SnsLookupRequest {
        network: request.network.clone(),
        source_endpoint: request.source_endpoint.clone(),
        now_unix_secs: request.now_unix_secs,
        input: request.input.clone(),
    };
    let (fetch_request, list, id, sns) = resolve_sns_lookup(&lookup_request, source)?;
    let cache_path =
        sns_neurons_cache_path(&request.icp_root, &request.network, &sns.root_canister_id);
    let lock_path =
        sns_neurons_refresh_lock_path(&request.icp_root, &request.network, &sns.root_canister_id);
    let attempt_path = sns_neurons_refresh_attempt_path(
        &request.icp_root,
        &request.network,
        &sns.root_canister_id,
    );
    let paths = SnsNeuronsCachePaths {
        cache_path,
        lock_path,
        attempt_path,
    };
    let cache_dir = paths
        .cache_path
        .parent()
        .expect("SNS neurons cache path always has parent");
    create_directory(cache_dir).map_err(sns_cache_file_error)?;
    let replaced_existing_cache = paths.cache_path.is_file();
    let context_paths = paths.clone();
    with_refresh_lock(
        RefreshLockRequest {
            lock_path: &paths.lock_path,
            target_path: &paths.cache_path,
            network: &request.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: SNS_NEURONS_REFRESH_LOCK_STALE_AFTER_SECONDS,
        },
        sns_cache_file_error,
        || {
            refresh_sns_neurons_cache_locked(
                SnsNeuronsRefreshContext {
                    request,
                    fetch_request: &fetch_request,
                    list,
                    id,
                    sns,
                    paths: context_paths,
                    replaced_existing_cache,
                },
                source,
            )
        },
    )
}

fn refresh_sns_neurons_cache_locked(
    context: SnsNeuronsRefreshContext<'_>,
    source: &dyn SnsNeuronsSource,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    write_sns_neurons_attempt(
        &context.paths.attempt_path,
        &attempt_from_parts(
            context.request,
            context.fetch_request,
            &context.sns,
            "running",
            0,
            0,
            None,
            None,
        ),
    )?;
    match fetch_complete_sns_neurons(
        context.request,
        context.fetch_request,
        &context.sns,
        source,
        &context.paths.attempt_path,
    ) {
        Ok(complete) => publish_complete_sns_neurons_cache(context, complete),
        Err(err) => {
            let _ = write_sns_neurons_attempt(
                &context.paths.attempt_path,
                &attempt_from_parts(
                    context.request,
                    context.fetch_request,
                    &context.sns,
                    "failed",
                    0,
                    0,
                    None,
                    Some(err.to_string()),
                ),
            );
            Err(err)
        }
    }
}

fn publish_complete_sns_neurons_cache(
    context: SnsNeuronsRefreshContext<'_>,
    complete: CompleteSnsNeurons,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    let cache = sns_neurons_cache_from_parts(
        &context.list,
        context.id,
        &context.sns,
        context.request.page_size,
        complete.page_count,
        complete.neurons,
    );
    let neuron_count = cache.neurons.len();
    let cache_json =
        serde_json::to_string_pretty(&cache).map_err(|source| SnsHostError::SerializeCache {
            path: context.paths.cache_path.clone(),
            source,
        })?;
    write_text_atomically(&context.paths.cache_path, &cache_json).map_err(sns_cache_file_error)?;
    write_sns_neurons_attempt(
        &context.paths.attempt_path,
        &attempt_from_parts(
            context.request,
            context.fetch_request,
            &context.sns,
            "complete",
            complete.page_count,
            neuron_count,
            complete.last_cursor,
            None,
        ),
    )?;
    Ok(SnsNeuronsRefreshReport {
        schema_version: SNS_NEURONS_REFRESH_REPORT_SCHEMA_VERSION,
        network: context.list.network,
        sns_wasm_canister_id: context.list.sns_wasm_canister_id,
        fetched_at: context.list.fetched_at,
        source_endpoint: context.list.source_endpoint,
        fetched_by: context.list.fetched_by,
        id: context.id,
        name: context.sns.name,
        root_canister_id: context.sns.root_canister_id,
        governance_canister_id: context.sns.governance_canister_id,
        cache_path: context.paths.cache_path.display().to_string(),
        refresh_lock_path: context.paths.lock_path.display().to_string(),
        refresh_attempt_path: context.paths.attempt_path.display().to_string(),
        page_size: context.request.page_size,
        page_count: complete.page_count,
        neuron_count,
        complete: true,
        replaced_existing_cache: context.replaced_existing_cache,
        wrote_cache: true,
    })
}

fn resolve_sns_lookup(
    request: &SnsLookupRequest,
    source: &dyn SnsListSource,
) -> Result<(SnsFetchRequest, MainnetSnsList, usize, MainnetSns), SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let fetch_request = fetch_request_from_parts(
        &request.source_endpoint,
        request.now_unix_secs,
        "ic-query".to_string(),
    );
    let mut list = source.fetch_deployed_snses(&fetch_request)?;
    assign_sns_ids_in_current_order(&mut list.sns_instances);
    sort_mainnet_sns_instances(&mut list.sns_instances, SnsListSort::Id);
    let (id, sns) = resolve_sns(&list.sns_instances, &request.input)?;
    Ok((fetch_request, list, id, sns))
}

#[must_use]
pub fn sns_list_report_text(report: &SnsListReport) -> String {
    let mut lines = Vec::new();
    lines.push(format!("network: {}", report.network));
    lines.push(format!(
        "sns_wasm_canister_id: {}",
        report.sns_wasm_canister_id
    ));
    lines.push(format!("sns_count: {}", report.sns_count));
    lines.push(format!("fetched_at: {}", report.fetched_at));
    lines.push(format!("source_endpoint: {}", report.source_endpoint));
    lines.push(format!("sort: {}", report.sort));
    lines.push(format!("metadata_errors: {}", report.metadata_error_count));
    if !report.sns_instances.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &[
                "ID",
                "NAME",
                "ROOT",
                "GOVERNANCE",
                "LEDGER",
                "SWAP",
                "INDEX",
            ],
            &report
                .sns_instances
                .iter()
                .map(|sns| {
                    [
                        sns.id.to_string(),
                        sns.name.clone(),
                        principal_for_list(&sns.root_canister_id, report.verbose),
                        principal_for_list(&sns.governance_canister_id, report.verbose),
                        principal_for_list(&sns.ledger_canister_id, report.verbose),
                        principal_for_list(&sns.swap_canister_id, report.verbose),
                        principal_for_list(&sns.index_canister_id, report.verbose),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Right,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
                ColumnAlign::Left,
            ],
        ));
    }
    if report.verbose && report.metadata_error_count > 0 {
        lines.push(String::new());
        lines.push("metadata_error_details:".to_string());
        for (governance_canister_id, error) in report.sns_instances.iter().filter_map(|sns| {
            sns.metadata_error
                .as_deref()
                .map(|error| (&sns.governance_canister_id, error))
        }) {
            lines.push(format!("- {governance_canister_id}: {error}"));
        }
    }
    lines.join("\n")
}

#[must_use]
pub fn sns_info_report_text(report: &SnsInfoReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!(
            "description: {}",
            optional_text(report.description.as_ref())
        ),
        format!("url: {}", optional_text(report.url.as_ref())),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("swap_canister_id: {}", report.swap_canister_id),
        format!("index_canister_id: {}", report.index_canister_id),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if let Some(error) = report.metadata_error.as_deref() {
        lines.push(format!("metadata_error: {error}"));
    }
    lines.join("\n")
}

#[must_use]
pub fn sns_token_report_text(report: &SnsTokenReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("ledger_canister_id: {}", report.ledger_canister_id),
        format!("sns_index_canister_id: {}", report.sns_index_canister_id),
        format!(
            "ledger_index_canister_id: {}",
            optional_text(report.ledger_index_canister_id.as_ref())
        ),
        format!("token_name: {}", report.token_name),
        format!("token_symbol: {}", report.token_symbol),
        format!("decimals: {}", report.decimals),
        format!(
            "transfer_fee: {}",
            base_units_decimal_text(&report.transfer_fee, report.decimals)
        ),
        format!(
            "total_supply: {}",
            base_units_decimal_text(&report.total_supply, report.decimals)
        ),
        format!(
            "minting_account_owner: {}",
            optional_text(report.minting_account_owner.as_ref())
        ),
        format!(
            "minting_account_subaccount_hex: {}",
            optional_text(report.minting_account_subaccount_hex.as_ref())
        ),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if let Some(error) = report.ledger_index_error.as_deref() {
        lines.push(format!("ledger_index_error: {error}"));
    }
    if !report.supported_standards.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["STANDARD", "URL"],
            &report
                .supported_standards
                .iter()
                .map(|standard| [standard.name.clone(), standard.url.clone()])
                .collect::<Vec<_>>(),
            &[ColumnAlign::Left, ColumnAlign::Left],
        ));
    }
    if !report.metadata.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &["METADATA", "TYPE", "VALUE"],
            &report
                .metadata
                .iter()
                .map(|row| {
                    [
                        row.key.clone(),
                        row.value_type.clone(),
                        truncate_text_value(
                            &token_metadata_value_text(row, report.decimals),
                            SNS_TOKEN_METADATA_TEXT_VALUE_LIMIT,
                        ),
                    ]
                })
                .collect::<Vec<_>>(),
            &[ColumnAlign::Left, ColumnAlign::Left, ColumnAlign::Left],
        ));
    }
    lines.join("\n")
}

#[must_use]
pub fn sns_params_report_text(report: &SnsParamsReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    lines.push(String::new());
    lines.push(render_table(
        &["PARAMETER", "VALUE"],
        &sns_params_text_rows(&report.parameters),
        &[ColumnAlign::Left, ColumnAlign::Right],
    ));
    lines.join("\n")
}

#[must_use]
pub fn sns_neurons_report_text(report: &SnsNeuronsReport) -> String {
    let mut lines = vec![
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("requested_limit: {}", report.requested_limit),
        format!(
            "owner_principal_id: {}",
            optional_text(report.owner_principal_id.as_ref())
        ),
        format!("verbose: {}", yes_no(report.verbose)),
        format!("data_source: {}", report.data_source),
        format!("sort: {}", report.sort),
        format!("total_neuron_count: {}", report.total_neuron_count),
        format!("neuron_count: {}", report.neuron_count),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ];
    if let Some(cache_path) = report.cache_path.as_deref() {
        lines.push(format!("cache_path: {cache_path}"));
    }
    if let Some(cache_complete) = report.cache_complete {
        lines.push(format!("cache_complete: {}", yes_no(cache_complete)));
    }
    if !report.neurons.is_empty() {
        lines.push(String::new());
        lines.push(render_table(
            &[
                "NEURON_ID",
                "STAKE",
                "MATURITY",
                "STAKED_MATURITY",
                "CREATED_AT",
            ],
            &report
                .neurons
                .iter()
                .map(|neuron| {
                    [
                        neuron_id_for_list(&neuron.neuron_id, report.verbose),
                        e8s_decimal_text(neuron.cached_neuron_stake_e8s),
                        e8s_decimal_text(neuron.maturity_e8s_equivalent),
                        optional_e8s_decimal_text(neuron.staked_maturity_e8s_equivalent),
                        neuron.created_at.clone(),
                    ]
                })
                .collect::<Vec<_>>(),
            &[
                ColumnAlign::Left,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Right,
                ColumnAlign::Left,
            ],
        ));
    }
    lines.join("\n")
}

#[must_use]
pub fn sns_neurons_refresh_report_text(report: &SnsNeuronsRefreshReport) -> String {
    [
        format!("network: {}", report.network),
        format!("sns_id: {}", report.id),
        format!("name: {}", report.name),
        format!("root_canister_id: {}", report.root_canister_id),
        format!("governance_canister_id: {}", report.governance_canister_id),
        format!("page_size: {}", report.page_size),
        format!("page_count: {}", report.page_count),
        format!("neuron_count: {}", report.neuron_count),
        format!("complete: {}", yes_no(report.complete)),
        format!("wrote_cache: {}", yes_no(report.wrote_cache)),
        format!(
            "replaced_existing_cache: {}",
            yes_no(report.replaced_existing_cache)
        ),
        format!("cache_path: {}", report.cache_path),
        format!("refresh_lock_path: {}", report.refresh_lock_path),
        format!("refresh_attempt_path: {}", report.refresh_attempt_path),
        format!("sns_wasm_canister_id: {}", report.sns_wasm_canister_id),
        format!("fetched_at: {}", report.fetched_at),
        format!("source_endpoint: {}", report.source_endpoint),
    ]
    .join("\n")
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsFetchRequest {
    endpoint: String,
    fetched_at: String,
    fetched_by: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MainnetSnsList {
    network: String,
    sns_wasm_canister_id: String,
    fetched_at: String,
    fetched_by: String,
    source_endpoint: String,
    sns_instances: Vec<MainnetSns>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MainnetSns {
    id: usize,
    name: String,
    description: Option<String>,
    url: Option<String>,
    root_canister_id: String,
    governance_canister_id: String,
    ledger_canister_id: String,
    swap_canister_id: String,
    index_canister_id: String,
    metadata_error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MainnetSnsCanisters {
    root_canister_id: String,
    governance_canister_id: String,
    ledger_canister_id: String,
    swap_canister_id: String,
    index_canister_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MainnetSnsToken {
    token_name: String,
    token_symbol: String,
    decimals: u8,
    transfer_fee: String,
    total_supply: String,
    minting_account_owner: Option<String>,
    minting_account_subaccount_hex: Option<String>,
    ledger_index_canister_id: Option<String>,
    ledger_index_error: Option<String>,
    supported_standards: Vec<SnsTokenStandardRow>,
    metadata: Vec<SnsTokenMetadataRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MainnetSnsNeurons {
    neurons: Vec<SnsNeuronRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MainnetSnsNeuronPage {
    neurons: Vec<SnsNeuronRow>,
    last_cursor: Option<SnsNeuronId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompleteSnsNeurons {
    neurons: Vec<SnsNeuronRow>,
    page_count: u32,
    last_cursor: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsNeuronsCachePaths {
    cache_path: PathBuf,
    lock_path: PathBuf,
    attempt_path: PathBuf,
}

struct SnsNeuronsRefreshContext<'a> {
    request: &'a SnsNeuronsRefreshRequest,
    fetch_request: &'a SnsFetchRequest,
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    paths: SnsNeuronsCachePaths,
    replaced_existing_cache: bool,
}

struct SnsNeuronsCachedReportParts {
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    requested_limit: u32,
    sort: SnsNeuronsSort,
    cache: SnsNeuronsCache,
    total_neuron_count: usize,
    cache_path: PathBuf,
    verbose: bool,
}

struct SnsNeuronsLiveReportParts {
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    requested_limit: u32,
    owner_principal_id: Option<String>,
    sort: SnsNeuronsSort,
    verbose: bool,
    neurons: MainnetSnsNeurons,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct ListDeployedSnsesRequest {}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct ListDeployedSnsesResponse {
    instances: Vec<DeployedSns>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct DeployedSns {
    root_canister_id: Option<Principal>,
    governance_canister_id: Option<Principal>,
    ledger_canister_id: Option<Principal>,
    swap_canister_id: Option<Principal>,
    index_canister_id: Option<Principal>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct GetMetadataRequest {}

#[derive(CandidType, Clone, Debug, Default, Deserialize, Eq, PartialEq)]
struct GetMetadataResponse {
    url: Option<String>,
    logo: Option<String>,
    name: Option<String>,
    description: Option<String>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcAccount {
    owner: Principal,
    subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
enum IcrcMetadataValue {
    Nat(Nat),
    Int(Int),
    Text(String),
    Blob(Vec<u8>),
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct IcrcSupportedStandard {
    name: String,
    url: String,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
enum GetIndexPrincipalResult {
    Ok(Principal),
    Err(GetIndexPrincipalError),
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
enum GetIndexPrincipalError {
    IndexPrincipalNotSet,
    GenericError {
        error_code: Nat,
        description: String,
    },
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct ListNeuronsRequest {
    of_principal: Option<Principal>,
    limit: u32,
    start_page_at: Option<SnsNeuronId>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct ListNeuronsResponse {
    neurons: Vec<SnsGovernanceNeuron>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct SnsNeuronId {
    id: Vec<u8>,
}

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
struct SnsGovernanceNeuron {
    id: Option<SnsNeuronId>,
    staked_maturity_e8s_equivalent: Option<u64>,
    maturity_e8s_equivalent: u64,
    cached_neuron_stake_e8s: u64,
    created_timestamp_seconds: u64,
}

trait SnsListSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError>;
}

trait SnsTokenSource: SnsListSource {
    fn fetch_sns_token(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsToken, SnsHostError>;
}

trait SnsParamsSource: SnsListSource {
    fn fetch_sns_params(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<SnsGovernanceParameters, SnsHostError>;
}

trait SnsNeuronsSource: SnsListSource {
    fn fetch_sns_neurons(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError>;

    fn fetch_sns_neuron_page(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError>;
}

struct LiveSnsListSource;

impl SnsListSource for LiveSnsListSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        fetch_mainnet_sns_list(request)
    }
}

impl SnsTokenSource for LiveSnsListSource {
    fn fetch_sns_token(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<MainnetSnsToken, SnsHostError> {
        fetch_mainnet_sns_token(request, sns)
    }
}

impl SnsParamsSource for LiveSnsListSource {
    fn fetch_sns_params(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
    ) -> Result<SnsGovernanceParameters, SnsHostError> {
        fetch_mainnet_sns_params(request, sns)
    }
}

impl SnsNeuronsSource for LiveSnsListSource {
    fn fetch_sns_neurons(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeurons, SnsHostError> {
        fetch_mainnet_sns_neurons(request, sns, limit, owner_principal_id)
    }

    fn fetch_sns_neuron_page(
        &self,
        request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        start_page_at: Option<&SnsNeuronId>,
        owner_principal_id: Option<&str>,
    ) -> Result<MainnetSnsNeuronPage, SnsHostError> {
        fetch_mainnet_sns_neuron_page(request, sns, limit, start_page_at, owner_principal_id)
    }
}

fn fetch_mainnet_sns_list(request: &SnsFetchRequest) -> Result<MainnetSnsList, SnsHostError> {
    block_on_current_thread(fetch_mainnet_sns_list_async(request)).map_err(SnsHostError::Runtime)?
}

fn fetch_mainnet_sns_token(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsToken, SnsHostError> {
    block_on_current_thread(fetch_mainnet_sns_token_async(request, sns))
        .map_err(SnsHostError::Runtime)?
}

fn fetch_mainnet_sns_params(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
) -> Result<SnsGovernanceParameters, SnsHostError> {
    block_on_current_thread(fetch_mainnet_sns_params_async(request, sns))
        .map_err(SnsHostError::Runtime)?
}

fn fetch_mainnet_sns_neurons(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    owner_principal_id: Option<&str>,
) -> Result<MainnetSnsNeurons, SnsHostError> {
    block_on_current_thread(fetch_mainnet_sns_neurons_async(
        request,
        sns,
        limit,
        owner_principal_id,
    ))
    .map_err(SnsHostError::Runtime)?
}

fn fetch_mainnet_sns_neuron_page(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    start_page_at: Option<&SnsNeuronId>,
    owner_principal_id: Option<&str>,
) -> Result<MainnetSnsNeuronPage, SnsHostError> {
    block_on_current_thread(fetch_mainnet_sns_neuron_page_async(
        request,
        sns,
        limit,
        start_page_at,
        owner_principal_id,
    ))
    .map_err(SnsHostError::Runtime)?
}

async fn fetch_mainnet_sns_list_async(
    request: &SnsFetchRequest,
) -> Result<MainnetSnsList, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let sns_wasm_canister =
        principal_from_text(MAINNET_SNS_WASM_CANISTER_ID, "sns_wasm_canister_id")?;
    let arg = Encode!(&ListDeployedSnsesRequest {}).map_err(|err| SnsHostError::CandidEncode {
        message: "ListDeployedSnsesRequest",
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(&sns_wasm_canister, "list_deployed_snses")
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method: "list_deployed_snses",
            reason: err.to_string(),
        })?;
    let response =
        Decode!(&bytes, ListDeployedSnsesResponse).map_err(|err| SnsHostError::CandidDecode {
            message: "ListDeployedSnsesResponse",
            reason: err.to_string(),
        })?;
    mainnet_sns_list_from_response(&agent, request, response).await
}

async fn fetch_mainnet_sns_token_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
) -> Result<MainnetSnsToken, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let ledger_canister = principal_from_text(&sns.ledger_canister_id, "ledger_canister_id")?;
    let token_name = query_ledger(&agent, &ledger_canister, "icrc1_name").await?;
    let token_symbol = query_ledger(&agent, &ledger_canister, "icrc1_symbol").await?;
    let decimals = query_ledger(&agent, &ledger_canister, "icrc1_decimals").await?;
    let transfer_fee: Nat = query_ledger(&agent, &ledger_canister, "icrc1_fee").await?;
    let total_supply: Nat = query_ledger(&agent, &ledger_canister, "icrc1_total_supply").await?;
    let minting_account: Option<IcrcAccount> =
        query_ledger(&agent, &ledger_canister, "icrc1_minting_account").await?;
    let supported_standards: Vec<IcrcSupportedStandard> =
        query_ledger(&agent, &ledger_canister, "icrc1_supported_standards").await?;
    let metadata: Vec<(String, IcrcMetadataValue)> =
        query_ledger(&agent, &ledger_canister, "icrc1_metadata").await?;
    let (ledger_index_canister_id, ledger_index_error) =
        match query_ledger::<GetIndexPrincipalResult>(
            &agent,
            &ledger_canister,
            "icrc106_get_index_principal",
        )
        .await
        {
            Ok(GetIndexPrincipalResult::Ok(principal)) => (Some(principal.to_text()), None),
            Ok(GetIndexPrincipalResult::Err(error)) => {
                (None, Some(index_principal_error_text(error)))
            }
            Err(error) => (None, Some(error.to_string())),
        };

    Ok(MainnetSnsToken {
        token_name,
        token_symbol,
        decimals,
        transfer_fee: transfer_fee.to_string(),
        total_supply: total_supply.to_string(),
        minting_account_owner: minting_account
            .as_ref()
            .map(|account| account.owner.to_text()),
        minting_account_subaccount_hex: minting_account
            .as_ref()
            .and_then(|account| account.subaccount.as_deref())
            .map(hex_bytes),
        ledger_index_canister_id,
        ledger_index_error,
        supported_standards: supported_standards
            .into_iter()
            .map(|standard| SnsTokenStandardRow {
                name: standard.name,
                url: standard.url,
            })
            .collect(),
        metadata: metadata
            .into_iter()
            .map(|(key, value)| metadata_row(key, value))
            .collect(),
    })
}

async fn fetch_mainnet_sns_params_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
) -> Result<SnsGovernanceParameters, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let governance_canister =
        principal_from_text(&sns.governance_canister_id, "governance_canister_id")?;
    let arg = Encode!(&()).map_err(|err| SnsHostError::CandidEncode {
        message: "get_nervous_system_parameters",
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(&governance_canister, "get_nervous_system_parameters")
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method: "get_nervous_system_parameters",
            reason: err.to_string(),
        })?;
    Decode!(&bytes, SnsGovernanceParameters).map_err(|err| SnsHostError::CandidDecode {
        message: "SnsGovernanceParameters",
        reason: err.to_string(),
    })
}

async fn fetch_mainnet_sns_neurons_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    owner_principal_id: Option<&str>,
) -> Result<MainnetSnsNeurons, SnsHostError> {
    let page =
        fetch_mainnet_sns_neuron_page_async(request, sns, limit, None, owner_principal_id).await?;
    Ok(MainnetSnsNeurons {
        neurons: page.neurons,
    })
}

async fn fetch_mainnet_sns_neuron_page_async(
    request: &SnsFetchRequest,
    sns: &MainnetSns,
    limit: u32,
    start_page_at: Option<&SnsNeuronId>,
    owner_principal_id: Option<&str>,
) -> Result<MainnetSnsNeuronPage, SnsHostError> {
    let agent = sns_agent(&request.endpoint)?;
    let governance_canister =
        principal_from_text(&sns.governance_canister_id, "governance_canister_id")?;
    let owner_principal = owner_principal_id
        .map(|principal| principal_from_text(principal, "owner_principal_id"))
        .transpose()?;
    let arg = Encode!(&ListNeuronsRequest {
        of_principal: owner_principal,
        limit,
        start_page_at: start_page_at.cloned(),
    })
    .map_err(|err| SnsHostError::CandidEncode {
        message: "ListNeuronsRequest",
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(&governance_canister, "list_neurons")
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method: "list_neurons",
            reason: err.to_string(),
        })?;
    let response =
        Decode!(&bytes, ListNeuronsResponse).map_err(|err| SnsHostError::CandidDecode {
            message: "ListNeuronsResponse",
            reason: err.to_string(),
        })?;
    let last_cursor = response.neurons.iter().rev().find_map(sns_neuron_cursor);
    Ok(MainnetSnsNeuronPage {
        neurons: response.neurons.into_iter().map(sns_neuron_row).collect(),
        last_cursor,
    })
}

async fn query_ledger<T>(
    agent: &Agent,
    ledger_canister: &Principal,
    method: &'static str,
) -> Result<T, SnsHostError>
where
    T: for<'de> Deserialize<'de> + CandidType,
{
    let arg = Encode!().map_err(|err| SnsHostError::CandidEncode {
        message: method,
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(ledger_canister, method)
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method,
            reason: err.to_string(),
        })?;
    candid::decode_one(&bytes).map_err(|err| SnsHostError::CandidDecode {
        message: method,
        reason: err.to_string(),
    })
}

fn sns_agent(endpoint: &str) -> Result<Agent, SnsHostError> {
    Agent::builder()
        .with_url(endpoint)
        .build()
        .map_err(|err| SnsHostError::AgentBuild {
            endpoint: endpoint.to_string(),
            reason: err.to_string(),
        })
}

async fn mainnet_sns_list_from_response(
    agent: &Agent,
    request: &SnsFetchRequest,
    response: ListDeployedSnsesResponse,
) -> Result<MainnetSnsList, SnsHostError> {
    let sns_canisters = response
        .instances
        .into_iter()
        .map(mainnet_sns_canisters_from_deployed_sns)
        .collect::<Result<Vec<_>, _>>()?;
    let fetched = stream::iter(
        sns_canisters
            .into_iter()
            .map(|sns| fetch_mainnet_sns_metadata(agent, sns)),
    )
    .buffered(SNS_METADATA_CONCURRENCY)
    .collect::<Vec<_>>()
    .await;
    let mut sns_instances = Vec::with_capacity(fetched.len());
    for sns in fetched {
        sns_instances.push(sns?);
    }
    Ok(MainnetSnsList {
        network: MAINNET_NETWORK.to_string(),
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: request.fetched_at.clone(),
        fetched_by: request.fetched_by.clone(),
        source_endpoint: request.endpoint.clone(),
        sns_instances,
    })
}

async fn fetch_mainnet_sns_metadata(
    agent: &Agent,
    sns: MainnetSnsCanisters,
) -> Result<MainnetSns, SnsHostError> {
    let governance_canister =
        principal_from_text(&sns.governance_canister_id, "governance_canister_id")?;
    let (metadata, metadata_error) =
        match fetch_governance_metadata(agent, &governance_canister).await {
            Ok(metadata) => (metadata, None),
            Err(err) => match metadata_error_summary(&err) {
                Some(summary) => (GetMetadataResponse::default(), Some(summary)),
                None => return Err(err),
            },
        };
    Ok(mainnet_sns_from_canisters_and_metadata(
        sns,
        metadata,
        metadata_error,
    ))
}

async fn fetch_governance_metadata(
    agent: &Agent,
    governance_canister: &Principal,
) -> Result<GetMetadataResponse, SnsHostError> {
    let arg = Encode!(&GetMetadataRequest {}).map_err(|err| SnsHostError::CandidEncode {
        message: "GetMetadataRequest",
        reason: err.to_string(),
    })?;
    let bytes = agent
        .query(governance_canister, "get_metadata")
        .with_arg(arg)
        .call()
        .await
        .map_err(|err| SnsHostError::AgentCall {
            method: "get_metadata",
            reason: err.to_string(),
        })?;
    let metadata =
        Decode!(&bytes, GetMetadataResponse).map_err(|err| SnsHostError::CandidDecode {
            message: "GetMetadataResponse",
            reason: err.to_string(),
        })?;
    Ok(metadata)
}

fn mainnet_sns_canisters_from_deployed_sns(
    sns: DeployedSns,
) -> Result<MainnetSnsCanisters, SnsHostError> {
    Ok(MainnetSnsCanisters {
        root_canister_id: required_principal_text(sns.root_canister_id, "root_canister_id")?,
        governance_canister_id: required_principal_text(
            sns.governance_canister_id,
            "governance_canister_id",
        )?,
        ledger_canister_id: required_principal_text(sns.ledger_canister_id, "ledger_canister_id")?,
        swap_canister_id: required_principal_text(sns.swap_canister_id, "swap_canister_id")?,
        index_canister_id: required_principal_text(sns.index_canister_id, "index_canister_id")?,
    })
}

fn mainnet_sns_from_canisters_and_metadata(
    sns: MainnetSnsCanisters,
    metadata: GetMetadataResponse,
    metadata_error: Option<String>,
) -> MainnetSns {
    let name = clean_optional_text(metadata.name)
        .unwrap_or_else(|| format!("unnamed-{}", short_principal(&sns.root_canister_id)));
    MainnetSns {
        id: 0,
        name,
        description: clean_optional_text(metadata.description),
        url: clean_optional_text(metadata.url),
        root_canister_id: sns.root_canister_id,
        governance_canister_id: sns.governance_canister_id,
        ledger_canister_id: sns.ledger_canister_id,
        swap_canister_id: sns.swap_canister_id,
        index_canister_id: sns.index_canister_id,
        metadata_error,
    }
}

fn required_principal_text(
    principal: Option<Principal>,
    field: &'static str,
) -> Result<String, SnsHostError> {
    principal
        .map(|principal| principal.to_text())
        .ok_or_else(|| SnsHostError::InvalidPrincipal {
            field,
            reason: "missing principal".to_string(),
        })
}

fn sns_list_report_from_list(
    list: MainnetSnsList,
    verbose: bool,
    sort: SnsListSort,
) -> SnsListReport {
    let MainnetSnsList {
        network,
        sns_wasm_canister_id,
        fetched_at,
        fetched_by,
        source_endpoint,
        sns_instances,
    } = list;
    let metadata_error_count = sns_instances
        .iter()
        .filter(|sns| sns.metadata_error.is_some())
        .count();
    let sns_instances = sns_instances
        .into_iter()
        .map(|sns| SnsListRow {
            id: sns.id,
            name: sns.name,
            root_canister_id: sns.root_canister_id,
            governance_canister_id: sns.governance_canister_id,
            ledger_canister_id: sns.ledger_canister_id,
            swap_canister_id: sns.swap_canister_id,
            index_canister_id: sns.index_canister_id,
            metadata_error: sns.metadata_error,
        })
        .collect::<Vec<_>>();
    SnsListReport {
        schema_version: SNS_LIST_REPORT_SCHEMA_VERSION,
        network,
        sns_wasm_canister_id,
        fetched_at,
        source_endpoint,
        fetched_by,
        verbose,
        sort: sort.as_str().to_string(),
        sns_count: sns_instances.len(),
        metadata_error_count,
        sns_instances,
    }
}

fn sns_info_report_from_list(list: MainnetSnsList, id: usize, sns: MainnetSns) -> SnsInfoReport {
    SnsInfoReport {
        schema_version: SNS_INFO_REPORT_SCHEMA_VERSION,
        network: list.network,
        sns_wasm_canister_id: list.sns_wasm_canister_id,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        id,
        name: sns.name,
        description: sns.description,
        url: sns.url,
        root_canister_id: sns.root_canister_id,
        governance_canister_id: sns.governance_canister_id,
        ledger_canister_id: sns.ledger_canister_id,
        swap_canister_id: sns.swap_canister_id,
        index_canister_id: sns.index_canister_id,
        metadata_error: sns.metadata_error,
    }
}

fn sns_token_report_from_parts(
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    token: MainnetSnsToken,
) -> SnsTokenReport {
    SnsTokenReport {
        schema_version: SNS_TOKEN_REPORT_SCHEMA_VERSION,
        network: list.network,
        sns_wasm_canister_id: list.sns_wasm_canister_id,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        id,
        name: sns.name,
        root_canister_id: sns.root_canister_id,
        ledger_canister_id: sns.ledger_canister_id,
        sns_index_canister_id: sns.index_canister_id,
        token_name: token.token_name,
        token_symbol: token.token_symbol,
        decimals: token.decimals,
        transfer_fee: token.transfer_fee,
        total_supply: token.total_supply,
        minting_account_owner: token.minting_account_owner,
        minting_account_subaccount_hex: token.minting_account_subaccount_hex,
        ledger_index_canister_id: token.ledger_index_canister_id,
        ledger_index_error: token.ledger_index_error,
        supported_standards: token.supported_standards,
        metadata: token.metadata,
    }
}

fn sns_params_report_from_parts(
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    parameters: SnsGovernanceParameters,
) -> SnsParamsReport {
    SnsParamsReport {
        schema_version: SNS_PARAMS_REPORT_SCHEMA_VERSION,
        network: list.network,
        sns_wasm_canister_id: list.sns_wasm_canister_id,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        id,
        name: sns.name,
        root_canister_id: sns.root_canister_id,
        governance_canister_id: sns.governance_canister_id,
        parameters,
    }
}

fn sns_neurons_report_from_parts(parts: SnsNeuronsLiveReportParts) -> SnsNeuronsReport {
    let neuron_count = parts.neurons.neurons.len();
    SnsNeuronsReport {
        schema_version: SNS_NEURONS_REPORT_SCHEMA_VERSION,
        network: parts.list.network,
        sns_wasm_canister_id: parts.list.sns_wasm_canister_id,
        fetched_at: parts.list.fetched_at,
        source_endpoint: parts.list.source_endpoint,
        fetched_by: parts.list.fetched_by,
        id: parts.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.root_canister_id,
        governance_canister_id: parts.sns.governance_canister_id,
        requested_limit: parts.requested_limit,
        owner_principal_id: parts.owner_principal_id,
        verbose: parts.verbose,
        data_source: "live".to_string(),
        sort: parts.sort.as_str().to_string(),
        cache_path: None,
        cache_complete: None,
        total_neuron_count: neuron_count,
        neuron_count,
        neurons: parts.neurons.neurons,
    }
}

fn sns_neurons_report_from_cache(parts: SnsNeuronsCachedReportParts) -> SnsNeuronsReport {
    let neuron_count = parts.cache.neurons.len();
    SnsNeuronsReport {
        schema_version: SNS_NEURONS_REPORT_SCHEMA_VERSION,
        network: parts.list.network,
        sns_wasm_canister_id: parts.list.sns_wasm_canister_id,
        fetched_at: parts.cache.fetched_at,
        source_endpoint: parts.cache.source_endpoint,
        fetched_by: parts.cache.fetched_by,
        id: parts.id,
        name: parts.sns.name,
        root_canister_id: parts.sns.root_canister_id,
        governance_canister_id: parts.sns.governance_canister_id,
        requested_limit: parts.requested_limit,
        owner_principal_id: None,
        verbose: parts.verbose,
        data_source: "cache".to_string(),
        sort: parts.sort.as_str().to_string(),
        cache_path: Some(parts.cache_path.display().to_string()),
        cache_complete: Some(parts.cache.completeness.status == "api_exhausted"),
        total_neuron_count: parts.total_neuron_count,
        neuron_count,
        neurons: parts.cache.neurons,
    }
}

fn sns_neurons_cache_from_parts(
    list: &MainnetSnsList,
    id: usize,
    sns: &MainnetSns,
    page_size: u32,
    page_count: u32,
    neurons: Vec<SnsNeuronRow>,
) -> SnsNeuronsCache {
    SnsNeuronsCache {
        schema_version: SNS_NEURONS_CACHE_SCHEMA_VERSION,
        network: list.network.clone(),
        sns_wasm_canister_id: list.sns_wasm_canister_id.clone(),
        fetched_at: list.fetched_at.clone(),
        source_endpoint: list.source_endpoint.clone(),
        fetched_by: list.fetched_by.clone(),
        id,
        name: sns.name.clone(),
        root_canister_id: sns.root_canister_id.clone(),
        governance_canister_id: sns.governance_canister_id.clone(),
        completeness: SnsNeuronsCompleteness {
            status: "api_exhausted".to_string(),
            page_size,
            page_count,
            row_count: neurons.len(),
            point_in_time_guaranteed: false,
        },
        neurons,
    }
}

fn fetch_complete_sns_neurons(
    request: &SnsNeuronsRefreshRequest,
    fetch_request: &SnsFetchRequest,
    sns: &MainnetSns,
    source: &dyn SnsNeuronsSource,
    attempt_path: &Path,
) -> Result<CompleteSnsNeurons, SnsHostError> {
    let mut neurons = Vec::new();
    let mut seen = HashSet::new();
    let mut page_count = 0_u32;
    let mut start_page_at: Option<SnsNeuronId> = None;
    let mut progress = ProgressLine::stderr();
    progress.update(&sns_neurons_progress_text(sns, page_count, neurons.len()));

    loop {
        if request
            .max_pages
            .is_some_and(|max_pages| page_count >= max_pages)
        {
            progress.finish(&format!(
                "{} stopped before completion",
                sns_neurons_progress_text(sns, page_count, neurons.len())
            ));
            return Err(SnsHostError::IncompleteRefresh {
                pages_fetched: page_count,
                rows_fetched: neurons.len(),
                reason: "max pages reached before API exhaustion".to_string(),
            });
        }
        let page = match source.fetch_sns_neuron_page(
            fetch_request,
            sns,
            request.page_size,
            start_page_at.as_ref(),
            None,
        ) {
            Ok(page) => page,
            Err(err) => {
                progress.finish(&format!(
                    "{} failed",
                    sns_neurons_progress_text(sns, page_count, neurons.len())
                ));
                return Err(err);
            }
        };
        page_count = page_count.saturating_add(1);
        let page_len = page.neurons.len();
        let next_cursor = page.last_cursor;
        let next_cursor_text = next_cursor.as_ref().map(|cursor| hex_bytes(&cursor.id));
        let mut new_rows = 0_usize;
        for neuron in page.neurons {
            if seen.insert(neuron.neuron_id.clone()) {
                new_rows = new_rows.saturating_add(1);
                neurons.push(neuron);
            }
        }
        write_sns_neurons_attempt(
            attempt_path,
            &attempt_from_parts(
                request,
                fetch_request,
                sns,
                "running",
                page_count,
                neurons.len(),
                next_cursor_text.clone(),
                None,
            ),
        )?;
        progress.update(&sns_neurons_progress_text(sns, page_count, neurons.len()));

        start_page_at.clone_from(&next_cursor);
        if page_len < usize::try_from(request.page_size).unwrap_or(usize::MAX)
            || next_cursor.is_none()
            || new_rows == 0
        {
            break;
        }
    }

    progress.finish(&format!(
        "{} complete",
        sns_neurons_progress_text(sns, page_count, neurons.len())
    ));
    Ok(CompleteSnsNeurons {
        neurons,
        page_count,
        last_cursor: start_page_at.as_ref().map(|cursor| hex_bytes(&cursor.id)),
    })
}

fn sns_neurons_progress_text(sns: &MainnetSns, pages: u32, rows: usize) -> String {
    format!(
        "refreshing SNS neurons for {}: pages={} rows={}",
        sns.name, pages, rows
    )
}

fn load_sns_neurons_cache(
    icp_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> Result<SnsNeuronsCache, SnsHostError> {
    let path = sns_neurons_cache_path(icp_root, network, root_canister_id);
    let cached: CachedJsonReport<SnsNeuronsCache> = load_json_cache(
        LoadJsonCacheRequest {
            path,
            network,
            expected_schema_version: SNS_NEURONS_CACHE_SCHEMA_VERSION,
        },
        LoadJsonCacheErrorHandlers {
            missing_cache: |path| SnsHostError::MissingNeuronsCache { path },
            read_cache: |path, source| SnsHostError::ReadCache { path, source },
            parse_cache: |path, source| SnsHostError::ParseCache { path, source },
            unsupported_schema: |version, expected| SnsHostError::UnsupportedCacheSchemaVersion {
                version,
                expected,
            },
            network_mismatch: |requested, actual| SnsHostError::CacheNetworkMismatch {
                requested,
                actual,
            },
        },
    )?;
    if cached.report.completeness.status != "api_exhausted" {
        return Err(SnsHostError::IncompleteRefresh {
            pages_fetched: cached.report.completeness.page_count,
            rows_fetched: cached.report.completeness.row_count,
            reason: "cached SNS neurons snapshot is not complete".to_string(),
        });
    }
    Ok(cached.report)
}

fn sort_sns_neurons(neurons: &mut [SnsNeuronRow], sort: SnsNeuronsSort) {
    match sort {
        SnsNeuronsSort::Api => {}
        SnsNeuronsSort::Id => neurons.sort_by(|left, right| left.neuron_id.cmp(&right.neuron_id)),
        SnsNeuronsSort::Stake => neurons.sort_by_key(|neuron| {
            (
                Reverse(neuron.cached_neuron_stake_e8s),
                neuron.neuron_id.clone(),
            )
        }),
        SnsNeuronsSort::Maturity => neurons.sort_by_key(|neuron| {
            (
                Reverse(neuron.maturity_e8s_equivalent),
                neuron.neuron_id.clone(),
            )
        }),
        SnsNeuronsSort::Created => neurons.sort_by_key(|neuron| {
            (
                Reverse(neuron.created_timestamp_seconds),
                neuron.neuron_id.clone(),
            )
        }),
    }
}

pub fn sns_neurons_cache_path(icp_root: &Path, network: &str, root_canister_id: &str) -> PathBuf {
    icp_root
        .join(".icq")
        .join("sns")
        .join(network)
        .join(root_canister_id)
        .join("neurons")
        .join("full.json")
}

pub fn sns_neurons_refresh_lock_path(
    icp_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    icp_root
        .join(".icq")
        .join("sns")
        .join(network)
        .join(root_canister_id)
        .join("neurons")
        .join("full.refresh.lock")
}

pub fn sns_neurons_refresh_attempt_path(
    icp_root: &Path,
    network: &str,
    root_canister_id: &str,
) -> PathBuf {
    icp_root
        .join(".icq")
        .join("sns")
        .join(network)
        .join(root_canister_id)
        .join("neurons")
        .join("full.refresh-attempt.json")
}

fn write_sns_neurons_attempt(
    path: &Path,
    attempt: &SnsNeuronsRefreshAttempt,
) -> Result<(), SnsHostError> {
    let data =
        serde_json::to_string_pretty(attempt).map_err(|source| SnsHostError::SerializeCache {
            path: path.to_path_buf(),
            source,
        })?;
    write_text_atomically(path, &data).map_err(sns_cache_file_error)
}

#[allow(clippy::too_many_arguments)]
fn attempt_from_parts(
    request: &SnsNeuronsRefreshRequest,
    fetch_request: &SnsFetchRequest,
    sns: &MainnetSns,
    status: &str,
    pages_fetched: u32,
    rows_fetched: usize,
    last_cursor: Option<String>,
    last_error: Option<String>,
) -> SnsNeuronsRefreshAttempt {
    SnsNeuronsRefreshAttempt {
        schema_version: SNS_NEURONS_REFRESH_ATTEMPT_SCHEMA_VERSION,
        network: request.network.clone(),
        source_endpoint: request.source_endpoint.clone(),
        started_at: fetch_request.fetched_at.clone(),
        updated_at: fetch_request.fetched_at.clone(),
        root_canister_id: sns.root_canister_id.clone(),
        governance_canister_id: sns.governance_canister_id.clone(),
        status: status.to_string(),
        page_size: request.page_size,
        pages_fetched,
        rows_fetched,
        last_cursor,
        last_error,
    }
}

fn assign_sns_ids_in_current_order(instances: &mut [MainnetSns]) {
    for (index, sns) in instances.iter_mut().enumerate() {
        sns.id = index + 1;
    }
}

fn sort_mainnet_sns_instances(instances: &mut [MainnetSns], sort: SnsListSort) {
    match sort {
        SnsListSort::Id => sort_mainnet_sns_instances_by_id(instances),
        SnsListSort::Name => instances.sort_by(|left, right| {
            left.name
                .to_lowercase()
                .cmp(&right.name.to_lowercase())
                .then_with(|| left.id.cmp(&right.id))
        }),
    }
}

fn sort_mainnet_sns_instances_by_id(instances: &mut [MainnetSns]) {
    instances.sort_by_key(|sns| sns.id);
}

fn resolve_sns(instances: &[MainnetSns], input: &str) -> Result<(usize, MainnetSns), SnsHostError> {
    if let Ok(id) = input.parse::<usize>() {
        return instances
            .iter()
            .find(|sns| sns.id == id)
            .cloned()
            .map(|sns| (id, sns))
            .ok_or(SnsHostError::UnknownSnsId {
                id,
                sns_count: instances.len(),
            });
    }

    let root_canister_id = Principal::from_text(input)
        .map_err(|_| SnsHostError::InvalidLookup {
            input: input.to_string(),
        })?
        .to_text();
    instances
        .iter()
        .find(|sns| sns.root_canister_id == root_canister_id)
        .map(|sns| (sns.id, sns.clone()))
        .ok_or(SnsHostError::UnknownSnsRoot { root_canister_id })
}

fn fetch_request_from_parts(
    source_endpoint: &str,
    now_unix_secs: u64,
    fetched_by: String,
) -> SnsFetchRequest {
    SnsFetchRequest {
        endpoint: source_endpoint.to_string(),
        fetched_at: format_utc_timestamp_secs(now_unix_secs),
        fetched_by,
    }
}

fn enforce_mainnet_network(network: &str) -> Result<(), SnsHostError> {
    if network == MAINNET_NETWORK {
        return Ok(());
    }
    Err(SnsHostError::UnsupportedNetwork {
        network: network.to_string(),
    })
}

fn principal_from_text(value: &str, field: &'static str) -> Result<Principal, SnsHostError> {
    Principal::from_text(value).map_err(|err| SnsHostError::InvalidPrincipal {
        field,
        reason: err.to_string(),
    })
}

fn sns_cache_file_error(err: CacheFileError) -> SnsHostError {
    SnsHostError::Cache(match err {
        CacheFileError::CreateDirectory { path, source } => {
            format!(
                "failed to create cache directory at {}: {source}",
                path.display()
            )
        }
        CacheFileError::CreateRefreshLock { path, source } => {
            format!(
                "failed to create refresh lock at {}: {source}",
                path.display()
            )
        }
        CacheFileError::ReadRefreshLock { path, source } => {
            format!(
                "failed to read refresh lock at {}: {source}",
                path.display()
            )
        }
        CacheFileError::ParseRefreshLock { path, source } => {
            format!(
                "failed to parse refresh lock at {}: {source}",
                path.display()
            )
        }
        CacheFileError::WriteRefreshLock { path, source } => {
            format!(
                "failed to write refresh lock at {}: {source}",
                path.display()
            )
        }
        CacheFileError::RemoveRefreshLock { path, source } => {
            format!(
                "failed to remove refresh lock at {}: {source}",
                path.display()
            )
        }
        CacheFileError::RefreshAlreadyInProgress {
            path,
            started_at_unix_ms,
        } => format!(
            "refresh already in progress; lock exists at {} since unix_ms={started_at_unix_ms}",
            path.display()
        ),
        CacheFileError::WriteTemp { path, source } => {
            format!(
                "failed to write cache temp file at {}: {source}",
                path.display()
            )
        }
        CacheFileError::SyncTemp { path, source } => {
            format!(
                "failed to sync cache temp file at {}: {source}",
                path.display()
            )
        }
        CacheFileError::Replace {
            temp_path,
            target_path,
            source,
        } => format!(
            "failed to replace cache at {} from {}: {source}",
            target_path.display(),
            temp_path.display()
        ),
        CacheFileError::SyncDirectory { path, source } => {
            format!(
                "failed to sync cache directory at {}: {source}",
                path.display()
            )
        }
        CacheFileError::WriteOutput { path, source } => {
            format!(
                "failed to write cache output at {}: {source}",
                path.display()
            )
        }
        CacheFileError::SyncOutput { path, source } => {
            format!(
                "failed to sync cache output at {}: {source}",
                path.display()
            )
        }
    })
}

fn principal_for_list(value: &str, verbose: bool) -> String {
    if verbose {
        value.to_string()
    } else {
        short_principal(value)
    }
}

fn short_principal(value: &str) -> String {
    value.chars().take(COMPACT_PRINCIPAL_CHARS).collect()
}

fn neuron_id_for_list(value: &str, verbose: bool) -> String {
    if verbose || value == "-" {
        value.to_string()
    } else {
        value.chars().take(COMPACT_NEURON_ID_CHARS).collect()
    }
}

fn optional_text(value: Option<&String>) -> &str {
    value.map_or("-", String::as_str)
}

fn sns_params_text_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    [
        sns_params_economic_rows(parameters),
        sns_params_delay_rows(parameters),
        sns_params_limit_rows(parameters),
        sns_params_permission_rows(parameters),
        sns_params_reward_rows(parameters),
    ]
    .concat()
}

fn sns_params_economic_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    vec![
        parameter_row(
            "neuron_minimum_stake",
            optional_e8s_text(parameters.neuron_minimum_stake_e8s),
        ),
        parameter_row(
            "transaction_fee",
            optional_e8s_text(parameters.transaction_fee_e8s),
        ),
        parameter_row("reject_cost", optional_e8s_text(parameters.reject_cost_e8s)),
    ]
}

fn sns_params_delay_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    vec![
        parameter_row(
            "max_dissolve_delay",
            optional_duration_text(parameters.max_dissolve_delay_seconds),
        ),
        parameter_row(
            "max_dissolve_delay_bonus",
            optional_percentage_text(parameters.max_dissolve_delay_bonus_percentage),
        ),
        parameter_row(
            "max_neuron_age_for_age_bonus",
            optional_duration_text(parameters.max_neuron_age_for_age_bonus),
        ),
        parameter_row(
            "max_age_bonus",
            optional_percentage_text(parameters.max_age_bonus_percentage),
        ),
        parameter_row(
            "initial_voting_period",
            optional_duration_text(parameters.initial_voting_period_seconds),
        ),
        parameter_row(
            "wait_for_quiet_deadline_increase",
            optional_duration_text(parameters.wait_for_quiet_deadline_increase_seconds),
        ),
        parameter_row(
            "minimum_dissolve_delay_to_vote",
            optional_duration_text(parameters.neuron_minimum_dissolve_delay_to_vote_seconds),
        ),
    ]
}

fn sns_params_limit_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    vec![
        parameter_row(
            "max_followees_per_function",
            optional_u64_text(parameters.max_followees_per_function),
        ),
        parameter_row(
            "max_proposals_to_keep_per_action",
            optional_u32_text(parameters.max_proposals_to_keep_per_action),
        ),
        parameter_row(
            "max_number_of_neurons",
            optional_u64_text(parameters.max_number_of_neurons),
        ),
        parameter_row(
            "max_number_of_proposals_with_ballots",
            optional_u64_text(parameters.max_number_of_proposals_with_ballots),
        ),
        parameter_row(
            "max_number_of_principals_per_neuron",
            optional_u64_text(parameters.max_number_of_principals_per_neuron),
        ),
        parameter_row(
            "maturity_modulation_disabled",
            optional_bool_text(parameters.maturity_modulation_disabled),
        ),
        parameter_row(
            "automatically_advance_target_version",
            optional_bool_text(parameters.automatically_advance_target_version),
        ),
    ]
}

fn sns_params_permission_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    vec![
        parameter_row(
            "neuron_claimer_permissions",
            optional_permissions_text(parameters.neuron_claimer_permissions.as_ref()),
        ),
        parameter_row(
            "neuron_grantable_permissions",
            optional_permissions_text(parameters.neuron_grantable_permissions.as_ref()),
        ),
    ]
}

fn sns_params_reward_rows(parameters: &SnsGovernanceParameters) -> Vec<[String; 2]> {
    let rewards = parameters.voting_rewards_parameters.as_ref();
    vec![
        parameter_row(
            "voting_reward_initial_rate",
            optional_basis_points_text(
                rewards.and_then(|rewards| rewards.initial_reward_rate_basis_points),
            ),
        ),
        parameter_row(
            "voting_reward_final_rate",
            optional_basis_points_text(
                rewards.and_then(|rewards| rewards.final_reward_rate_basis_points),
            ),
        ),
        parameter_row(
            "voting_reward_transition_duration",
            optional_duration_text(
                rewards.and_then(|rewards| rewards.reward_rate_transition_duration_seconds),
            ),
        ),
        parameter_row(
            "voting_reward_round_duration",
            optional_duration_text(rewards.and_then(|rewards| rewards.round_duration_seconds)),
        ),
        parameter_row(
            "additional_critical_native_actions",
            parameters.custom_proposal_criticality.as_ref().map_or_else(
                || "-".to_string(),
                |criticality| comma_join_u64(&criticality.additional_critical_native_action_ids),
            ),
        ),
    ]
}

fn parameter_row(name: &str, value: String) -> [String; 2] {
    [name.to_string(), value]
}

fn optional_e8s_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), e8s_decimal_text)
}

fn optional_e8s_decimal_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), e8s_decimal_text)
}

fn optional_duration_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), duration_text)
}

fn optional_percentage_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| format!("{value}%"))
}

fn optional_basis_points_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), basis_points_text)
}

fn optional_u64_text(value: Option<u64>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

fn optional_u32_text(value: Option<u32>) -> String {
    value.map_or_else(|| "-".to_string(), |value| value.to_string())
}

fn optional_bool_text(value: Option<bool>) -> String {
    value.map_or_else(|| "-".to_string(), |value| yes_no(value).to_string())
}

fn optional_permissions_text(value: Option<&SnsNeuronPermissionList>) -> String {
    value.map_or_else(
        || "-".to_string(),
        |permissions| {
            permissions
                .permissions
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        },
    )
}

fn comma_join_u64(values: &[u64]) -> String {
    if values.is_empty() {
        return "-".to_string();
    }
    values
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn basis_points_text(value: u64) -> String {
    let whole = value / 100;
    let fractional = value % 100;
    format!("{whole}.{fractional:02}%")
}

fn duration_text(seconds: u64) -> String {
    const MINUTE: u64 = 60;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;

    if seconds == 0 {
        "0s".to_string()
    } else if seconds >= DAY {
        scaled_duration_unit_text(seconds, DAY, "d")
    } else if seconds >= HOUR {
        scaled_duration_unit_text(seconds, HOUR, "h")
    } else if seconds >= MINUTE {
        scaled_duration_unit_text(seconds, MINUTE, "m")
    } else {
        format!("{seconds}s")
    }
}

fn scaled_duration_unit_text(seconds: u64, unit_seconds: u64, suffix: &str) -> String {
    if seconds.is_multiple_of(unit_seconds) {
        return format!("{}{suffix}", seconds / unit_seconds);
    }
    let hundredths =
        ((u128::from(seconds) * 100) + (u128::from(unit_seconds) / 2)) / u128::from(unit_seconds);
    let whole = hundredths / 100;
    let fractional = hundredths % 100;
    format!("{whole}.{fractional:02}{suffix}")
}

fn clean_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn sns_neuron_row(neuron: SnsGovernanceNeuron) -> SnsNeuronRow {
    SnsNeuronRow {
        neuron_id: neuron
            .id
            .map_or_else(|| "-".to_string(), |id| hex_bytes(&id.id)),
        cached_neuron_stake_e8s: neuron.cached_neuron_stake_e8s,
        maturity_e8s_equivalent: neuron.maturity_e8s_equivalent,
        staked_maturity_e8s_equivalent: neuron.staked_maturity_e8s_equivalent,
        created_timestamp_seconds: neuron.created_timestamp_seconds,
        created_at: format_utc_timestamp_secs(neuron.created_timestamp_seconds),
    }
}

fn sns_neuron_cursor(neuron: &SnsGovernanceNeuron) -> Option<SnsNeuronId> {
    neuron.id.clone()
}

fn metadata_row(key: String, value: IcrcMetadataValue) -> SnsTokenMetadataRow {
    if key == SNS_TOKEN_LOGO_METADATA_KEY {
        return SnsTokenMetadataRow {
            key,
            value_type: "bool".to_string(),
            value: JsonValue::Bool(metadata_value_is_present(&value)),
        };
    }

    let (value_type, value) = match value {
        IcrcMetadataValue::Nat(value) => ("nat", value.to_string()),
        IcrcMetadataValue::Int(value) => ("int", value.to_string()),
        IcrcMetadataValue::Text(value) => ("text", value),
        IcrcMetadataValue::Blob(value) => ("blob", hex_bytes(&value)),
    };
    SnsTokenMetadataRow {
        key,
        value_type: value_type.to_string(),
        value: JsonValue::String(value),
    }
}

fn metadata_value_is_present(value: &IcrcMetadataValue) -> bool {
    match value {
        IcrcMetadataValue::Text(value) => !value.trim().is_empty(),
        IcrcMetadataValue::Blob(value) => !value.is_empty(),
        IcrcMetadataValue::Nat(_) | IcrcMetadataValue::Int(_) => true,
    }
}

fn index_principal_error_text(error: GetIndexPrincipalError) -> String {
    match error {
        GetIndexPrincipalError::IndexPrincipalNotSet => "index principal not set".to_string(),
        GetIndexPrincipalError::GenericError {
            error_code,
            description,
        } => format!("generic error {error_code}: {description}"),
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn truncate_text_value(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    let mut truncated = value.chars().take(limit).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn metadata_value_text(value: &JsonValue) -> String {
    match value {
        JsonValue::String(value) => value.clone(),
        JsonValue::Bool(value) => value.to_string(),
        JsonValue::Number(value) => value.to_string(),
        JsonValue::Null => "-".to_string(),
        JsonValue::Array(_) | JsonValue::Object(_) => value.to_string(),
    }
}

fn token_metadata_value_text(row: &SnsTokenMetadataRow, decimals: u8) -> String {
    let value = metadata_value_text(&row.value);
    if row.key == "icrc1:fee" {
        base_units_decimal_text(&value, decimals)
    } else {
        value
    }
}

fn metadata_error_summary(err: &SnsHostError) -> Option<String> {
    match err {
        SnsHostError::AgentCall { method, reason } => Some(format!("{method}: {reason}")),
        SnsHostError::CandidEncode { message, reason } => {
            Some(format!("encode {message}: {reason}"))
        }
        SnsHostError::CandidDecode { message, reason } => {
            Some(format!("decode {message}: {reason}"))
        }
        SnsHostError::UnsupportedNetwork { .. }
        | SnsHostError::Runtime(_)
        | SnsHostError::AgentBuild { .. }
        | SnsHostError::InvalidPrincipal { .. }
        | SnsHostError::UnknownSnsId { .. }
        | SnsHostError::UnknownSnsRoot { .. }
        | SnsHostError::InvalidLookup { .. }
        | SnsHostError::MissingNeuronsCache { .. }
        | SnsHostError::ReadCache { .. }
        | SnsHostError::ParseCache { .. }
        | SnsHostError::SerializeCache { .. }
        | SnsHostError::UnsupportedCacheSchemaVersion { .. }
        | SnsHostError::CacheNetworkMismatch { .. }
        | SnsHostError::Cache(_)
        | SnsHostError::IncompleteRefresh { .. }
        | SnsHostError::MissingCacheRoot => None,
    }
}

#[cfg(test)]
#[path = "report_tests.rs"]
mod tests;
