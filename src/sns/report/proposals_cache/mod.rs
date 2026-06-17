//! Module: sns::report::proposals_cache
//!
//! Responsibility: complete SNS proposal snapshot refresh and inspection.
//! Does not own: command parsing, live proposal conversion, or text rendering.
//! Boundary: stores complete proposal snapshots and refresh-attempt metadata.

mod attempt;
mod collection;
mod errors;
mod model;
mod paths;
mod reports;
mod storage;

use crate::snapshot_cache::{
    LockedSnapshotRefreshRequest, SnapshotCompleteness, run_snapshot_refresh_with_attempts,
    with_locked_snapshot_refresh, write_snapshot_json,
};
use crate::sns::report::lookup::{
    enforce_mainnet_network, lookup_request_from_parts, resolve_sns_lookup,
};
use crate::sns::report::source::{MainnetSns, MainnetSnsList, SnsFetchRequest, SnsProposalsSource};
use crate::sns::report::{
    SnsHostError, SnsProposalRow, SnsProposalsRefreshReport, SnsProposalsRefreshRequest,
    live::LiveSnsSource,
};
use attempt::{
    SnsProposalsAttemptContext, SnsProposalsAttemptProgress, write_complete_attempt,
    write_failed_attempt, write_starting_attempt,
};
use collection::fetch_complete_sns_proposals;
use errors::sns_cache_file_error;
use model::{
    CompleteSnsProposals, SnsProposalsCache, SnsProposalsCacheMetadata, SnsProposalsCacheRows,
};
use paths::SnsProposalsCachePaths;
pub(in crate::sns::report) use reports::build_sns_proposals_report_from_cache_or_refresh;
pub use reports::{build_sns_proposals_cache_list_report, build_sns_proposals_cache_status_report};

pub(super) const SNS_PROPOSALS_CACHE_SCHEMA_VERSION: u32 = 1;
pub(super) const SNS_PROPOSALS_CACHE_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub(super) const SNS_PROPOSALS_CACHE_STATUS_REPORT_SCHEMA_VERSION: u32 = 1;
pub(super) const SNS_PROPOSALS_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;

const SNS_PROPOSALS_REFRESH_LOCK_STALE_AFTER_SECONDS: u64 = 30 * 60;
const SNS_PROPOSALS_AUTO_REFRESH_PAGE_SIZE: u32 = 100;

///
/// SnsProposalsRefreshContext
///
/// Resolved context for one locked proposal snapshot refresh.
///

struct SnsProposalsRefreshContext<'a> {
    request: &'a SnsProposalsRefreshRequest,
    fetch_request: &'a SnsFetchRequest,
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    paths: SnsProposalsCachePaths,
    replaced_existing_cache: bool,
}

impl SnsProposalsRefreshContext<'_> {
    fn attempt_context(&self) -> SnsProposalsAttemptContext<'_> {
        SnsProposalsAttemptContext {
            path: &self.paths.attempt_path,
            request: self.request,
            fetch_request: self.fetch_request,
            sns: &self.sns,
        }
    }
}

/// Refresh a complete SNS proposal snapshot using the live SNS source.
pub fn refresh_sns_proposals_cache(
    request: &SnsProposalsRefreshRequest,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    refresh_sns_proposals_cache_with_source(request, &LiveSnsSource)
}

/// Refresh a complete SNS proposal snapshot using an injected source.
pub(in crate::sns::report) fn refresh_sns_proposals_cache_with_source(
    request: &SnsProposalsRefreshRequest,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let lookup_request = lookup_request_from_parts(
        &request.network,
        &request.source_endpoint,
        request.now_unix_secs,
        &request.input,
    );
    let lookup = resolve_sns_lookup(&lookup_request, source)?;
    let paths = SnsProposalsCachePaths::for_root(
        &request.icp_root,
        &request.network,
        &lookup.sns.root_canister_id,
    );
    let context_paths = paths.clone();
    let fetch_request = lookup.fetch_request;
    let list = lookup.list;
    let id = lookup.id;
    let sns = lookup.sns;
    with_locked_snapshot_refresh(
        LockedSnapshotRefreshRequest {
            snapshot_path: &paths.cache_path,
            refresh_lock_path: &paths.lock_path,
            network: &request.network,
            now_unix_secs: request.now_unix_secs,
            lock_stale_after_seconds: SNS_PROPOSALS_REFRESH_LOCK_STALE_AFTER_SECONDS,
        },
        sns_cache_file_error,
        |refresh_state| {
            refresh_sns_proposals_cache_locked(
                SnsProposalsRefreshContext {
                    request,
                    fetch_request: &fetch_request,
                    list,
                    id,
                    sns,
                    paths: context_paths,
                    replaced_existing_cache: refresh_state.replaced_existing_snapshot,
                },
                source,
            )
        },
    )
}

fn refresh_sns_proposals_cache_locked(
    context: SnsProposalsRefreshContext<'_>,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    run_snapshot_refresh_with_attempts(
        || write_starting_attempt(context.attempt_context()),
        || {
            let complete = fetch_complete_sns_proposals(
                context.request,
                context.fetch_request,
                &context.sns,
                source,
                &context.paths.attempt_path,
            )?;
            publish_complete_sns_proposals_cache(&context, complete)
        },
        |err| write_failed_attempt(context.attempt_context(), err),
    )
}

fn publish_complete_sns_proposals_cache(
    context: &SnsProposalsRefreshContext<'_>,
    complete: CompleteSnsProposals,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    let CompleteSnsProposals {
        proposals,
        page_count,
        last_cursor,
    } = complete;
    let cache = sns_proposals_cache_from_parts(
        &context.list,
        context.id,
        &context.sns,
        context.request.page_size,
        page_count,
        proposals,
    );
    let proposal_count = cache.data.proposals.len();
    write_snapshot_json(
        &context.paths.cache_path,
        &cache,
        |path, source| SnsHostError::SerializeCache { path, source },
        sns_cache_file_error,
    )?;
    write_complete_attempt(
        context.attempt_context(),
        SnsProposalsAttemptProgress::new(page_count, proposal_count, last_cursor),
    )?;
    Ok(SnsProposalsRefreshReport {
        schema_version: SNS_PROPOSALS_REFRESH_REPORT_SCHEMA_VERSION,
        network: context.list.network.clone(),
        sns_wasm_canister_id: context.list.sns_wasm_canister_id.clone(),
        fetched_at: context.list.fetched_at.clone(),
        source_endpoint: context.list.source_endpoint.clone(),
        fetched_by: context.list.fetched_by.clone(),
        id: context.id,
        name: context.sns.name.clone(),
        root_canister_id: context.sns.root_canister_id.clone(),
        governance_canister_id: context.sns.governance_canister_id.clone(),
        cache_path: context.paths.cache_path.display().to_string(),
        refresh_lock_path: context.paths.lock_path.display().to_string(),
        refresh_attempt_path: context.paths.attempt_path.display().to_string(),
        page_size: context.request.page_size,
        page_count,
        proposal_count,
        complete: true,
        replaced_existing_cache: context.replaced_existing_cache,
        wrote_cache: true,
    })
}

fn sns_proposals_cache_from_parts(
    list: &MainnetSnsList,
    id: usize,
    sns: &MainnetSns,
    page_size: u32,
    page_count: u32,
    proposals: Vec<SnsProposalRow>,
) -> SnsProposalsCache {
    SnsProposalsCache {
        schema_version: SNS_PROPOSALS_CACHE_SCHEMA_VERSION,
        network: list.network.clone(),
        fetched_at: list.fetched_at.clone(),
        source_endpoint: list.source_endpoint.clone(),
        fetched_by: list.fetched_by.clone(),
        metadata: SnsProposalsCacheMetadata {
            sns_wasm_canister_id: list.sns_wasm_canister_id.clone(),
            id,
            name: sns.name.clone(),
            root_canister_id: sns.root_canister_id.clone(),
            governance_canister_id: sns.governance_canister_id.clone(),
        },
        completeness: SnapshotCompleteness::api_exhausted(
            page_size,
            page_count,
            proposals.len(),
            false,
        ),
        data: SnsProposalsCacheRows { proposals },
    }
}
