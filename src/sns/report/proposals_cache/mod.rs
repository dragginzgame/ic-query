//! Module: sns::report::proposals_cache
//!
//! Responsibility: complete SNS proposal snapshot refresh and inspection.
//! Does not own: command parsing, live proposal conversion, or text rendering.
//! Boundary: stores complete proposal snapshots and refresh-attempt metadata.

use crate::cache_file::{CacheFileError, LoadJsonCacheErrorMapper, LoadJsonCacheRequest};
use crate::snapshot_cache::{
    LockedSnapshotRefreshRequest, PagedCollectionPage, PagedCollectionState, PagedSnapshotRefresh,
    SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION, SnapshotCompleteness, SnapshotEnvelope,
    SnapshotHeader, SnapshotJsonPaths, SnapshotKey, SnapshotRefreshAttempt, load_complete_snapshot,
    load_snapshot_header, read_snapshot_refresh_attempt, run_paged_snapshot_refresh,
    run_snapshot_refresh_with_attempts, snapshot_network_dir, with_locked_snapshot_refresh,
    write_snapshot_json, write_snapshot_refresh_attempt,
};
use crate::sns::report::lookup::{
    enforce_mainnet_network, lookup_request_from_parts, resolve_sns_lookup,
};
use crate::sns::report::source::{
    MainnetSns, MainnetSnsList, MainnetSnsProposalPage, MainnetSnsProposals, SnsFetchRequest,
    SnsProposalsSource,
};
use crate::sns::report::{
    SnsHostError, SnsProposalRow, SnsProposalStatusFilter, SnsProposalsCacheListReport,
    SnsProposalsCacheListRequest, SnsProposalsCacheStatusReport, SnsProposalsCacheStatusRequest,
    SnsProposalsCacheSummary, SnsProposalsRefreshAttemptStatus, SnsProposalsRefreshReport,
    SnsProposalsRefreshRequest, SnsProposalsReport, SnsProposalsRequest,
    assemble::{SnsProposalsReportParts, sns_proposals_report_from_parts},
    live::LiveSnsSource,
};
use candid::Principal;
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::path::{Path, PathBuf};

pub(super) const SNS_PROPOSALS_CACHE_SCHEMA_VERSION: u32 = 1;
pub(super) const SNS_PROPOSALS_CACHE_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
pub(super) const SNS_PROPOSALS_CACHE_STATUS_REPORT_SCHEMA_VERSION: u32 = 1;
pub(super) const SNS_PROPOSALS_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;

const SNS_PROPOSALS_REFRESH_LOCK_STALE_AFTER_SECONDS: u64 = 30 * 60;
const SNS_PROPOSALS_AUTO_REFRESH_PAGE_SIZE: u32 = 100;

type SnsProposalsCache = SnapshotEnvelope<SnsProposalsCacheMetadata, SnsProposalsCacheRows>;
type SnsProposalsCacheHeader = SnapshotHeader<SnsProposalsCacheHeaderMetadata>;
type SnsProposalsRefreshAttempt = SnapshotRefreshAttempt<SnsProposalsRefreshAttemptMetadata>;

struct SnsProposalsCacheErrors;

impl LoadJsonCacheErrorMapper for SnsProposalsCacheErrors {
    type Error = SnsHostError;

    fn missing_cache(&self, path: PathBuf) -> Self::Error {
        SnsHostError::MissingProposalsCache { path }
    }

    fn read_cache(&self, path: PathBuf, source: std::io::Error) -> Self::Error {
        SnsHostError::ReadCache { path, source }
    }

    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error {
        SnsHostError::ParseCache { path, source }
    }

    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error {
        SnsHostError::UnsupportedCacheSchemaVersion { version, expected }
    }

    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error {
        SnsHostError::CacheNetworkMismatch { requested, actual }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
struct SnsProposalsCacheMetadata {
    sns_wasm_canister_id: String,
    id: usize,
    name: String,
    root_canister_id: String,
    governance_canister_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
struct SnsProposalsCacheRows {
    proposals: Vec<SnsProposalRow>,
}

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize)]
struct SnsProposalsCacheHeaderMetadata {
    id: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
struct SnsProposalsRefreshAttemptMetadata {
    root_canister_id: String,
    governance_canister_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompleteSnsProposals {
    proposals: Vec<SnsProposalRow>,
    page_count: u32,
    last_cursor: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsProposalsCachePaths {
    cache_path: PathBuf,
    lock_path: PathBuf,
    attempt_path: PathBuf,
}

impl SnsProposalsCachePaths {
    fn for_root(icp_root: &Path, network: &str, root_canister_id: &str) -> Self {
        let snapshot_key = SnapshotKey::full("sns", network, root_canister_id, "proposals");
        let snapshot_paths = SnapshotJsonPaths::for_key(icp_root, &snapshot_key);
        Self {
            cache_path: snapshot_paths.snapshot_path,
            lock_path: snapshot_paths.refresh_lock_path,
            attempt_path: snapshot_paths.refresh_attempt_path,
        }
    }
}

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

#[derive(Clone, Copy)]
struct SnsProposalsAttemptContext<'a> {
    path: &'a Path,
    request: &'a SnsProposalsRefreshRequest,
    fetch_request: &'a SnsFetchRequest,
    sns: &'a MainnetSns,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SnsProposalsAttemptProgress {
    pages_fetched: u32,
    rows_fetched: usize,
    last_cursor: Option<String>,
}

impl SnsProposalsAttemptProgress {
    const fn new(pages_fetched: u32, rows_fetched: usize, last_cursor: Option<String>) -> Self {
        Self {
            pages_fetched,
            rows_fetched,
            last_cursor,
        }
    }

    const fn starting() -> Self {
        Self {
            pages_fetched: 0,
            rows_fetched: 0,
            last_cursor: None,
        }
    }
}

pub fn refresh_sns_proposals_cache(
    request: &SnsProposalsRefreshRequest,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    refresh_sns_proposals_cache_with_source(request, &LiveSnsSource)
}

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

pub fn build_sns_proposals_cache_list_report(
    request: &SnsProposalsCacheListRequest,
) -> Result<SnsProposalsCacheListReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let cache_root = sns_network_cache_dir(&request.icp_root, &request.network);
    let mut caches = list_sns_proposals_cache_summaries(&request.icp_root, &request.network)?;
    caches.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.root_canister_id.cmp(&right.root_canister_id))
    });
    Ok(SnsProposalsCacheListReport {
        schema_version: SNS_PROPOSALS_CACHE_LIST_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root: cache_root.display().to_string(),
        cache_count: caches.len(),
        caches,
    })
}

pub fn build_sns_proposals_cache_status_report(
    request: &SnsProposalsCacheStatusRequest,
) -> Result<SnsProposalsCacheStatusReport, SnsHostError> {
    enforce_mainnet_network(&request.network)?;
    let cache_root = sns_network_cache_dir(&request.icp_root, &request.network);
    if let Ok(id) = request.input.parse::<usize>() {
        return build_id_cache_status_report(request, cache_root.display().to_string(), id);
    }
    build_root_cache_status_report(request, cache_root.display().to_string())
}

pub(in crate::sns::report) fn build_sns_proposals_report_from_cache_or_refresh(
    request: &SnsProposalsRequest,
    icp_root: &Path,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsReport, SnsHostError> {
    let cache = load_or_refresh_sns_proposals_cache(request, icp_root, source)?;
    Ok(sns_proposals_report_from_cache(request, cache))
}

fn load_or_refresh_sns_proposals_cache(
    request: &SnsProposalsRequest,
    icp_root: &Path,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsCache, SnsHostError> {
    if let Ok(id) = request.input.parse::<usize>() {
        if let Some((_path, cache)) =
            find_sns_proposals_cache_by_id(icp_root, &request.network, id)?
        {
            return Ok(cache);
        }
        return refresh_and_load_sns_proposals_cache(request, icp_root, source);
    }

    let root_canister_id = Principal::from_text(&request.input)
        .map_err(|_| SnsHostError::InvalidLookup {
            input: request.input.clone(),
        })?
        .to_text();
    let paths = SnsProposalsCachePaths::for_root(icp_root, &request.network, &root_canister_id);
    if paths.cache_path.is_file() {
        return load_sns_proposals_cache_at(paths.cache_path, &request.network);
    }
    refresh_and_load_sns_proposals_cache(request, icp_root, source)
}

fn refresh_and_load_sns_proposals_cache(
    request: &SnsProposalsRequest,
    icp_root: &Path,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsCache, SnsHostError> {
    let refresh = refresh_sns_proposals_cache_with_source(
        &SnsProposalsRefreshRequest {
            network: request.network.clone(),
            source_endpoint: request.source_endpoint.clone(),
            now_unix_secs: request.now_unix_secs,
            input: request.input.clone(),
            icp_root: icp_root.to_path_buf(),
            page_size: SNS_PROPOSALS_AUTO_REFRESH_PAGE_SIZE,
            max_pages: None,
        },
        source,
    )?;
    load_sns_proposals_cache_at(PathBuf::from(refresh.cache_path), &request.network)
}

fn sns_proposals_report_from_cache(
    request: &SnsProposalsRequest,
    cache: SnsProposalsCache,
) -> SnsProposalsReport {
    let metadata = cache.metadata;
    let list = MainnetSnsList {
        network: cache.network,
        sns_wasm_canister_id: metadata.sns_wasm_canister_id.clone(),
        fetched_at: cache.fetched_at,
        fetched_by: cache.fetched_by,
        source_endpoint: cache.source_endpoint,
        sns_instances: Vec::new(),
    };
    let sns = MainnetSns {
        id: metadata.id,
        name: metadata.name,
        description: None,
        url: None,
        root_canister_id: metadata.root_canister_id,
        governance_canister_id: metadata.governance_canister_id,
        ledger_canister_id: String::new(),
        swap_canister_id: String::new(),
        index_canister_id: String::new(),
        metadata_error: None,
    };
    let proposals = cache
        .data
        .proposals
        .into_iter()
        .filter(|proposal| proposal_matches_before(proposal, request.before_proposal_id))
        .filter(|proposal| proposal_matches_status(proposal, request.status))
        .take(usize::try_from(request.limit).unwrap_or(usize::MAX))
        .collect::<Vec<_>>();
    sns_proposals_report_from_parts(SnsProposalsReportParts {
        list,
        id: sns.id,
        sns,
        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status: request.status,
        topic: request.topic,
        verbose: request.verbose,
        proposals: MainnetSnsProposals { proposals },
    })
}

fn proposal_matches_before(proposal: &SnsProposalRow, before_proposal_id: Option<u64>) -> bool {
    before_proposal_id.is_none_or(|before| {
        proposal
            .proposal_id
            .is_some_and(|proposal_id| proposal_id < before)
    })
}

fn proposal_matches_status(proposal: &SnsProposalRow, status: SnsProposalStatusFilter) -> bool {
    match status {
        SnsProposalStatusFilter::Any => true,
        SnsProposalStatusFilter::Open => proposal.decision_state == "open",
        SnsProposalStatusFilter::Executed => proposal.decision_state == "executed",
        SnsProposalStatusFilter::Failed => proposal.decision_state == "failed",
        SnsProposalStatusFilter::Rejected | SnsProposalStatusFilter::Adopted => false,
    }
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

fn fetch_complete_sns_proposals(
    request: &SnsProposalsRefreshRequest,
    fetch_request: &SnsFetchRequest,
    sns: &MainnetSns,
    source: &dyn SnsProposalsSource,
    attempt_path: &Path,
) -> Result<CompleteSnsProposals, SnsHostError> {
    run_paged_snapshot_refresh(SnsProposalsRefreshPages {
        request,
        fetch_request,
        sns,
        source,
        attempt_path,
        state: SnsProposalsCollectionState::new(),
    })
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

struct SnsProposalsRefreshPages<'a> {
    request: &'a SnsProposalsRefreshRequest,
    fetch_request: &'a SnsFetchRequest,
    sns: &'a MainnetSns,
    source: &'a dyn SnsProposalsSource,
    attempt_path: &'a Path,
    state: SnsProposalsCollectionState,
}

impl PagedSnapshotRefresh for SnsProposalsRefreshPages<'_> {
    type Complete = CompleteSnsProposals;
    type Error = SnsHostError;

    fn progress_text(&self) -> String {
        format!(
            "refreshing SNS proposals for {}: pages={} rows={}",
            self.sns.name,
            self.state.page_count(),
            self.state.row_count()
        )
    }

    fn max_pages_reached(&self) -> bool {
        self.request
            .max_pages
            .is_some_and(|max_pages| self.state.page_count() >= max_pages)
    }

    fn incomplete_refresh_error(&self) -> Self::Error {
        SnsHostError::IncompleteRefresh {
            pages_fetched: self.state.page_count(),
            rows_fetched: self.state.row_count(),
            reason: "max pages reached before API exhaustion".to_string(),
        }
    }

    fn fetch_next_page(&mut self) -> Result<PagedCollectionPage, Self::Error> {
        let page = self.source.fetch_sns_proposal_page(
            self.fetch_request,
            self.sns,
            self.request.page_size,
            self.state.before_proposal_id(),
        )?;
        Ok(self.state.ingest_page(page))
    }

    fn write_running_attempt(&self, page: &PagedCollectionPage) -> Result<(), Self::Error> {
        write_running_attempt(
            SnsProposalsAttemptContext {
                path: self.attempt_path,
                request: self.request,
                fetch_request: self.fetch_request,
                sns: self.sns,
            },
            SnsProposalsAttemptProgress::new(
                self.state.page_count(),
                self.state.row_count(),
                page.last_cursor_text.clone(),
            ),
        )
    }

    fn page_exhausts_collection(&self, page: &PagedCollectionPage) -> bool {
        page.exhausts_collection(self.request.page_size, self.state.has_next_cursor())
    }

    fn into_complete(self) -> Self::Complete {
        self.state.into_complete()
    }
}

struct SnsProposalsCollectionState {
    pages: PagedCollectionState<SnsProposalRow, u64>,
}

impl SnsProposalsCollectionState {
    fn new() -> Self {
        Self {
            pages: PagedCollectionState::new(),
        }
    }

    const fn page_count(&self) -> u32 {
        self.pages.page_count()
    }

    const fn row_count(&self) -> usize {
        self.pages.row_count()
    }

    const fn before_proposal_id(&self) -> Option<u64> {
        match self.pages.next_cursor() {
            Some(cursor) => Some(*cursor),
            None => None,
        }
    }

    const fn has_next_cursor(&self) -> bool {
        self.pages.has_next_cursor()
    }

    fn ingest_page(&mut self, page: MainnetSnsProposalPage) -> PagedCollectionPage {
        self.pages.ingest_page(
            page.proposals,
            page.last_cursor,
            ToString::to_string,
            proposal_row_id,
        )
    }

    fn into_complete(self) -> CompleteSnsProposals {
        let complete = self.pages.into_complete(ToString::to_string);
        CompleteSnsProposals {
            proposals: complete.rows,
            page_count: complete.page_count,
            last_cursor: complete.last_cursor,
        }
    }
}

fn proposal_row_id(proposal: &SnsProposalRow) -> String {
    proposal.proposal_id.map_or_else(
        || {
            format!(
                "missing:{}:{}",
                proposal.proposal_creation_timestamp_seconds, proposal.title
            )
        },
        |proposal_id| proposal_id.to_string(),
    )
}

fn write_starting_attempt(context: SnsProposalsAttemptContext<'_>) -> Result<(), SnsHostError> {
    write_attempt_status(
        context,
        "running",
        SnsProposalsAttemptProgress::starting(),
        None,
    )
}

fn write_running_attempt(
    context: SnsProposalsAttemptContext<'_>,
    progress: SnsProposalsAttemptProgress,
) -> Result<(), SnsHostError> {
    write_attempt_status(context, "running", progress, None)
}

fn write_complete_attempt(
    context: SnsProposalsAttemptContext<'_>,
    progress: SnsProposalsAttemptProgress,
) -> Result<(), SnsHostError> {
    write_attempt_status(context, "complete", progress, None)
}

fn write_failed_attempt(context: SnsProposalsAttemptContext<'_>, err: &SnsHostError) {
    let _ = write_attempt_status(
        context,
        "failed",
        SnsProposalsAttemptProgress::starting(),
        Some(err.to_string()),
    );
}

fn write_attempt_status(
    context: SnsProposalsAttemptContext<'_>,
    status: &'static str,
    progress: SnsProposalsAttemptProgress,
    last_error: Option<String>,
) -> Result<(), SnsHostError> {
    let attempt = SnsProposalsRefreshAttempt {
        schema_version: SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION,
        network: context.request.network.clone(),
        source_endpoint: context.request.source_endpoint.clone(),
        started_at: context.fetch_request.fetched_at.clone(),
        updated_at: context.fetch_request.fetched_at.clone(),
        metadata: SnsProposalsRefreshAttemptMetadata {
            root_canister_id: context.sns.root_canister_id.clone(),
            governance_canister_id: context.sns.governance_canister_id.clone(),
        },
        status: status.to_string(),
        page_size: context.request.page_size,
        pages_fetched: progress.pages_fetched,
        rows_fetched: progress.rows_fetched,
        last_cursor: progress.last_cursor,
        last_error,
    };
    write_snapshot_refresh_attempt(
        context.path,
        &attempt,
        |path, source| SnsHostError::SerializeCache { path, source },
        sns_cache_file_error,
    )
}

fn build_id_cache_status_report(
    request: &SnsProposalsCacheStatusRequest,
    cache_root: String,
    id: usize,
) -> Result<SnsProposalsCacheStatusReport, SnsHostError> {
    let cache = find_sns_proposals_cache_by_id(&request.icp_root, &request.network, id)?
        .map(|(path, cache)| sns_proposals_cache_summary(path, cache));
    let refresh_attempt_path = cache
        .as_ref()
        .map(|cache| cache.refresh_attempt_path.clone());
    let latest_attempt = cache
        .as_ref()
        .and_then(|cache| cache.latest_attempt.clone());
    Ok(cache_status_report(
        request,
        cache_root,
        cache,
        None,
        refresh_attempt_path,
        latest_attempt,
    ))
}

fn build_root_cache_status_report(
    request: &SnsProposalsCacheStatusRequest,
    cache_root: String,
) -> Result<SnsProposalsCacheStatusReport, SnsHostError> {
    let root_canister_id = Principal::from_text(&request.input)
        .map_err(|_| SnsHostError::InvalidLookup {
            input: request.input.clone(),
        })?
        .to_text();
    let paths =
        SnsProposalsCachePaths::for_root(&request.icp_root, &request.network, &root_canister_id);
    let cache = if paths.cache_path.is_file() {
        Some(sns_proposals_cache_summary(
            paths.cache_path.clone(),
            load_sns_proposals_cache_at(paths.cache_path.clone(), &request.network)?,
        ))
    } else {
        None
    };
    let latest_attempt = cache.as_ref().map_or_else(
        || read_sns_proposals_attempt_status(&paths.attempt_path),
        |cache| cache.latest_attempt.clone(),
    );
    Ok(cache_status_report(
        request,
        cache_root,
        cache,
        Some(paths.cache_path.display().to_string()),
        Some(paths.attempt_path.display().to_string()),
        latest_attempt,
    ))
}

fn cache_status_report(
    request: &SnsProposalsCacheStatusRequest,
    cache_root: String,
    cache: Option<SnsProposalsCacheSummary>,
    expected_cache_path: Option<String>,
    refresh_attempt_path: Option<String>,
    latest_attempt: Option<SnsProposalsRefreshAttemptStatus>,
) -> SnsProposalsCacheStatusReport {
    SnsProposalsCacheStatusReport {
        schema_version: SNS_PROPOSALS_CACHE_STATUS_REPORT_SCHEMA_VERSION,
        network: request.network.clone(),
        cache_root,
        input: request.input.clone(),
        found: cache.is_some(),
        cache,
        expected_cache_path,
        refresh_attempt_path,
        latest_attempt,
    }
}

fn list_sns_proposals_cache_summaries(
    icp_root: &Path,
    network: &str,
) -> Result<Vec<SnsProposalsCacheSummary>, SnsHostError> {
    let cache_root = sns_network_cache_dir(icp_root, network);
    if !cache_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut summaries = Vec::new();
    for entry in std::fs::read_dir(&cache_root).map_err(|source| SnsHostError::ReadCache {
        path: cache_root.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| SnsHostError::ReadCache {
            path: cache_root.clone(),
            source,
        })?;
        let root_path = entry.path();
        let cache_path = root_path.join("proposals").join("full.json");
        if cache_path.is_file() {
            summaries.push(sns_proposals_cache_summary(
                cache_path.clone(),
                load_sns_proposals_cache_at(cache_path, network)?,
            ));
        }
    }
    Ok(summaries)
}

fn find_sns_proposals_cache_by_id(
    icp_root: &Path,
    network: &str,
    id: usize,
) -> Result<Option<(PathBuf, SnsProposalsCache)>, SnsHostError> {
    let cache_root = sns_network_cache_dir(icp_root, network);
    if !cache_root.is_dir() {
        return Ok(None);
    }
    for entry in std::fs::read_dir(&cache_root).map_err(|source| SnsHostError::ReadCache {
        path: cache_root.clone(),
        source,
    })? {
        let entry = entry.map_err(|source| SnsHostError::ReadCache {
            path: cache_root.clone(),
            source,
        })?;
        let cache_path = entry.path().join("proposals").join("full.json");
        if !cache_path.is_file() {
            continue;
        }
        let header = load_sns_proposals_cache_header(cache_path.clone(), network)?;
        if header.metadata.id == id {
            let cache = load_sns_proposals_cache_at(cache_path.clone(), network)?;
            return Ok(Some((cache_path, cache)));
        }
    }
    Ok(None)
}

fn sns_proposals_cache_summary(
    cache_path: PathBuf,
    cache: SnsProposalsCache,
) -> SnsProposalsCacheSummary {
    let latest_attempt =
        read_sns_proposals_attempt_status(&attempt_path_for_cache_path(&cache_path));
    SnsProposalsCacheSummary {
        id: cache.metadata.id,
        name: cache.metadata.name,
        root_canister_id: cache.metadata.root_canister_id,
        governance_canister_id: cache.metadata.governance_canister_id,
        complete: cache.completeness.is_api_exhausted(),
        row_count: cache.completeness.row_count,
        page_count: cache.completeness.page_count,
        page_size: cache.completeness.page_size,
        fetched_at: cache.fetched_at,
        source_endpoint: cache.source_endpoint,
        refresh_attempt_path: attempt_path_for_cache_path(&cache_path)
            .display()
            .to_string(),
        cache_path: cache_path.display().to_string(),
        latest_attempt,
    }
}

fn load_sns_proposals_cache_at(
    cache_path: PathBuf,
    network: &str,
) -> Result<SnsProposalsCache, SnsHostError> {
    load_complete_snapshot(
        LoadJsonCacheRequest {
            path: cache_path,
            network,
            expected_schema_version: SNS_PROPOSALS_CACHE_SCHEMA_VERSION,
        },
        SnsProposalsCacheErrors,
        |completeness| SnsHostError::IncompleteRefresh {
            pages_fetched: completeness.page_count,
            rows_fetched: completeness.row_count,
            reason: "cached SNS proposals snapshot is not complete".to_string(),
        },
    )
}

fn load_sns_proposals_cache_header(
    cache_path: PathBuf,
    network: &str,
) -> Result<SnsProposalsCacheHeader, SnsHostError> {
    load_snapshot_header(
        LoadJsonCacheRequest {
            path: cache_path,
            network,
            expected_schema_version: SNS_PROPOSALS_CACHE_SCHEMA_VERSION,
        },
        SnsProposalsCacheErrors,
    )
}

fn read_sns_proposals_attempt_status(path: &Path) -> Option<SnsProposalsRefreshAttemptStatus> {
    let attempt = read_snapshot_refresh_attempt::<SnsProposalsRefreshAttempt>(path)?;
    Some(SnsProposalsRefreshAttemptStatus {
        status: attempt.status,
        started_at: attempt.started_at,
        updated_at: attempt.updated_at,
        page_size: attempt.page_size,
        pages_fetched: attempt.pages_fetched,
        rows_fetched: attempt.rows_fetched,
        last_cursor: attempt.last_cursor,
        last_error: attempt.last_error,
    })
}

fn sns_network_cache_dir(icp_root: &Path, network: &str) -> PathBuf {
    snapshot_network_dir(icp_root, "sns", network)
}

fn attempt_path_for_cache_path(cache_path: &Path) -> PathBuf {
    cache_path.with_file_name("full.refresh-attempt.json")
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
        CacheFileError::SerializeRefreshLock { path, source } => {
            format!(
                "failed to serialize refresh lock at {}: {source}",
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
