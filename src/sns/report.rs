use crate::{
    cache_file::{
        CacheFileError, CachedJsonReport, JsonCacheReport, LoadJsonCacheErrorHandlers,
        LoadJsonCacheRequest, RefreshLockRequest, create_directory, load_json_cache,
        with_refresh_lock, write_text_atomically,
    },
    ic_registry::DEFAULT_MAINNET_ENDPOINT,
    progress::ProgressLine,
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};
use candid::Principal;
use live::LiveSnsListSource;
pub use model::*;
use serde::{Deserialize as SerdeDeserialize, Serialize};
use source::{
    MainnetSns, MainnetSnsCanisters, MainnetSnsList, MainnetSnsNeuronPage, MainnetSnsNeurons,
    MainnetSnsToken, SnsFetchRequest, SnsListSource, SnsNeuronId, SnsNeuronsSource,
    SnsParamsSource, SnsTokenSource,
};
use std::{
    cmp::Reverse,
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

mod live;
mod model;
mod source;
mod text;

pub use text::{
    sns_info_report_text, sns_list_report_text, sns_neurons_refresh_report_text,
    sns_neurons_report_text, sns_params_report_text, sns_token_report_text,
};

#[cfg(test)]
use live::{IcrcMetadataValue, metadata_row};

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
const SNS_TOKEN_LOGO_METADATA_KEY: &str = "icrc1:logo";
const SNS_METADATA_CONCURRENCY: usize = 16;

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

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
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
        &attempt_from_parts(SnsNeuronsAttemptParts {
            request: context.request,
            fetch_request: context.fetch_request,
            sns: &context.sns,
            status: "running",
            pages_fetched: 0,
            rows_fetched: 0,
            last_cursor: None,
            last_error: None,
        }),
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
                &failed_attempt_from_latest_progress(&context, &err),
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
        &attempt_from_parts(SnsNeuronsAttemptParts {
            request: context.request,
            fetch_request: context.fetch_request,
            sns: &context.sns,
            status: "complete",
            pages_fetched: complete.page_count,
            rows_fetched: neuron_count,
            last_cursor: complete.last_cursor,
            last_error: None,
        }),
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

struct SnsNeuronsAttemptParts<'a> {
    request: &'a SnsNeuronsRefreshRequest,
    fetch_request: &'a SnsFetchRequest,
    sns: &'a MainnetSns,
    status: &'static str,
    pages_fetched: u32,
    rows_fetched: usize,
    last_cursor: Option<String>,
    last_error: Option<String>,
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
            &attempt_from_parts(SnsNeuronsAttemptParts {
                request,
                fetch_request,
                sns,
                status: "running",
                pages_fetched: page_count,
                rows_fetched: neurons.len(),
                last_cursor: next_cursor_text.clone(),
                last_error: None,
            }),
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

fn failed_attempt_from_latest_progress(
    context: &SnsNeuronsRefreshContext<'_>,
    err: &SnsHostError,
) -> SnsNeuronsRefreshAttempt {
    let latest = read_sns_neurons_attempt(&context.paths.attempt_path);
    let pages_fetched = latest.as_ref().map_or(0, |attempt| attempt.pages_fetched);
    let rows_fetched = latest.as_ref().map_or(0, |attempt| attempt.rows_fetched);
    let last_cursor = latest.and_then(|attempt| attempt.last_cursor);
    attempt_from_parts(SnsNeuronsAttemptParts {
        request: context.request,
        fetch_request: context.fetch_request,
        sns: &context.sns,
        status: "failed",
        pages_fetched,
        rows_fetched,
        last_cursor,
        last_error: Some(err.to_string()),
    })
}

fn read_sns_neurons_attempt(path: &Path) -> Option<SnsNeuronsRefreshAttempt> {
    fs::read(path)
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
}

fn attempt_from_parts(parts: SnsNeuronsAttemptParts<'_>) -> SnsNeuronsRefreshAttempt {
    SnsNeuronsRefreshAttempt {
        schema_version: SNS_NEURONS_REFRESH_ATTEMPT_SCHEMA_VERSION,
        network: parts.request.network.clone(),
        source_endpoint: parts.request.source_endpoint.clone(),
        started_at: parts.fetch_request.fetched_at.clone(),
        updated_at: current_timestamp_text(&parts.fetch_request.fetched_at),
        root_canister_id: parts.sns.root_canister_id.clone(),
        governance_canister_id: parts.sns.governance_canister_id.clone(),
        status: parts.status.to_string(),
        page_size: parts.request.page_size,
        pages_fetched: parts.pages_fetched,
        rows_fetched: parts.rows_fetched,
        last_cursor: parts.last_cursor,
        last_error: parts.last_error,
    }
}

fn current_timestamp_text(fallback: &str) -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or_else(
        |_| fallback.to_string(),
        |duration| format_utc_timestamp_secs(duration.as_secs()),
    )
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

pub(super) fn short_principal(value: &str) -> String {
    value.chars().take(COMPACT_PRINCIPAL_CHARS).collect()
}

pub(super) fn hex_bytes(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[cfg(test)]
#[path = "report_tests.rs"]
mod tests;
