mod attempt;
mod errors;
mod model;
mod paths;
mod storage;

use super::{
    SNS_NEURONS_REPORT_SCHEMA_VERSION, SnsHostError, SnsNeuronRow, SnsNeuronsCacheListReport,
    SnsNeuronsCacheListRequest, SnsNeuronsCacheStatusReport, SnsNeuronsCacheStatusRequest,
    SnsNeuronsRefreshAttemptStatus, SnsNeuronsRefreshReport, SnsNeuronsRefreshRequest,
    SnsNeuronsReport, SnsNeuronsRequest, SnsNeuronsSort, enforce_mainnet_network, hex_bytes,
    lookup_request_from_parts, resolve_sns_lookup,
};
use super::{
    live::LiveSnsSource,
    source::{MainnetSns, MainnetSnsList, SnsFetchRequest, SnsNeuronId, SnsNeuronsSource},
};
use crate::{
    cache_file::{RefreshLockRequest, create_directory, with_refresh_lock, write_text_atomically},
    progress::ProgressLine,
};
use candid::Principal;
use std::{collections::HashSet, path::Path};

use attempt::{
    SnsNeuronsAttemptParts, attempt_from_parts, failed_attempt_from_latest_progress,
    read_sns_neurons_attempt_status, write_sns_neurons_attempt,
};
use errors::sns_cache_file_error;
use model::{
    CompleteSnsNeurons, SnsNeuronsCache, SnsNeuronsCachedReportParts, SnsNeuronsCompleteness,
};
use paths::{SnsNeuronsCachePaths, sns_network_cache_dir};
pub(super) use paths::{
    sns_neurons_cache_path, sns_neurons_refresh_attempt_path, sns_neurons_refresh_lock_path,
};
use storage::{
    find_sns_neurons_cache_by_id, list_sns_neurons_cache_summaries, load_sns_neurons_cache_at,
    load_sns_neurons_cache_for_input, sns_neurons_cache_summary, sort_sns_neurons,
};

pub(super) const SNS_NEURONS_CACHE_SCHEMA_VERSION: u32 = 1;
pub(super) const SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub(super) const SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION: u32 = 1;
pub(super) const SNS_NEURONS_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
const SNS_NEURONS_REFRESH_LOCK_STALE_AFTER_SECONDS: u64 = 30 * 60;

struct SnsNeuronsRefreshContext<'a> {
    request: &'a SnsNeuronsRefreshRequest,
    fetch_request: &'a SnsFetchRequest,
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    paths: SnsNeuronsCachePaths,
    replaced_existing_cache: bool,
}

pub fn build_sns_neurons_cache_list_report(
    request: &SnsNeuronsCacheListRequest,
) -> Result<SnsNeuronsCacheListReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let cache_root = sns_network_cache_dir(&request.icp_root, &request.network);
    let mut caches = list_sns_neurons_cache_summaries(&request.icp_root, &request.network)?;
    caches.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.root_canister_id.cmp(&right.root_canister_id))
    });
    Ok(SnsNeuronsCacheListReport {
        schema_version: SNS_NEURONS_CACHE_LIST_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root: cache_root.display().to_string(),
        cache_count: caches.len(),
        caches,
    })
}

pub fn build_sns_neurons_cache_status_report(
    request: &SnsNeuronsCacheStatusRequest,
) -> Result<SnsNeuronsCacheStatusReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let cache_root = sns_network_cache_dir(&request.icp_root, &request.network);
    if let Ok(id) = request.input.parse::<usize>() {
        let cache = find_sns_neurons_cache_by_id(&request.icp_root, &request.network, id)?
            .map(|(path, cache)| sns_neurons_cache_summary(path, cache));
        let refresh_attempt_path = cache
            .as_ref()
            .map(|cache| cache.refresh_attempt_path.clone());
        let latest_attempt = cache
            .as_ref()
            .and_then(|cache| cache.latest_attempt.clone());
        return Ok(SnsNeuronsCacheStatusReport {
            schema_version: SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
            network: request.network.clone(),
            cache_root: cache_root.display().to_string(),
            input: request.input.clone(),
            found: cache.is_some(),
            cache,
            expected_cache_path: None,
            refresh_attempt_path,
            latest_attempt,
        });
    }

    let root_canister_id = Principal::from_text(&request.input)
        .map_err(|_| SnsHostError::InvalidLookup {
            input: request.input.clone(),
        })?
        .to_text();
    let cache_path = sns_neurons_cache_path(&request.icp_root, &request.network, &root_canister_id);
    let attempt_path =
        sns_neurons_refresh_attempt_path(&request.icp_root, &request.network, &root_canister_id);
    let cache = if cache_path.is_file() {
        Some(sns_neurons_cache_summary(
            cache_path.clone(),
            load_sns_neurons_cache_at(cache_path.clone(), &request.network)?,
        ))
    } else {
        None
    };
    let latest_attempt = cache.as_ref().map_or_else(
        || read_sns_neurons_attempt_status(&attempt_path),
        |cache| cache.latest_attempt.clone(),
    );
    Ok(SnsNeuronsCacheStatusReport {
        schema_version: SNS_NEURONS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root: cache_root.display().to_string(),
        input: request.input.clone(),
        found: cache.is_some(),
        cache,
        expected_cache_path: Some(cache_path.display().to_string()),
        refresh_attempt_path: Some(attempt_path.display().to_string()),
        latest_attempt,
    })
}

pub(super) fn build_sns_neurons_report_from_cache(
    request: &SnsNeuronsRequest,
) -> Result<SnsNeuronsReport, SnsHostError> {
    let icp_root = request
        .icp_root
        .as_ref()
        .ok_or(SnsHostError::MissingCacheRoot)?;
    let (cache_path, mut cache) =
        load_sns_neurons_cache_for_input(icp_root, &request.network, &request.input)?;
    sort_sns_neurons(&mut cache.neurons, request.sort);
    let total_neuron_count = cache.neurons.len();
    let limit = usize::try_from(request.limit).unwrap_or(usize::MAX);
    cache.neurons.truncate(limit);
    Ok(sns_neurons_report_from_cache(SnsNeuronsCachedReportParts {
        requested_limit: request.limit,
        sort: request.sort,
        cache,
        total_neuron_count,
        cache_path,
        verbose: request.verbose,
    }))
}

pub fn refresh_sns_neurons_cache(
    request: &SnsNeuronsRefreshRequest,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    refresh_sns_neurons_cache_with_source(request, &LiveSnsSource)
}

pub(super) fn refresh_sns_neurons_cache_with_source(
    request: &SnsNeuronsRefreshRequest,
    source: &dyn SnsNeuronsSource,
) -> Result<SnsNeuronsRefreshReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let lookup_request = lookup_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        &request.input,
    );
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
                &failed_attempt_from_latest_progress(
                    &context.paths.attempt_path,
                    context.request,
                    context.fetch_request,
                    &context.sns,
                    &err,
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

fn sns_neurons_report_from_cache(parts: SnsNeuronsCachedReportParts) -> SnsNeuronsReport {
    let cache = parts.cache;
    let neuron_count = cache.neurons.len();
    let cache_complete = cache.completeness.status == "api_exhausted";
    SnsNeuronsReport {
        schema_version: SNS_NEURONS_REPORT_SCHEMA_VERSION,
        network: cache.network,
        sns_wasm_canister_id: cache.sns_wasm_canister_id,
        fetched_at: cache.fetched_at,
        source_endpoint: cache.source_endpoint,
        fetched_by: cache.fetched_by,
        id: cache.id,
        name: cache.name,
        root_canister_id: cache.root_canister_id,
        governance_canister_id: cache.governance_canister_id,
        requested_limit: parts.requested_limit,
        owner_principal_id: None,
        verbose: parts.verbose,
        data_source: "cache".to_string(),
        sort: parts.sort.as_str().to_string(),
        cache_path: Some(parts.cache_path.display().to_string()),
        cache_complete: Some(cache_complete),
        total_neuron_count: parts.total_neuron_count,
        neuron_count,
        neurons: cache.neurons,
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
