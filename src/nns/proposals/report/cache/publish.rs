//! Module: nns::proposals::report::cache::publish
//!
//! Responsibility: publish complete NNS proposal snapshots.
//! Does not own: refresh locking, live proposal paging, or command parsing.
//! Boundary: writes complete cache JSON and complete-attempt metadata.

use super::{
    NNS_PROPOSAL_CACHE_SCHEMA_VERSION, NNS_PROPOSAL_REFRESH_REPORT_SCHEMA_VERSION,
    attempt::{NnsProposalAttemptProgress, write_complete_attempt},
    model::{
        CompleteNnsProposalCollection, NnsProposalCache, NnsProposalCacheMetadata,
        NnsProposalCacheRows, NnsProposalRefreshReport, NnsProposalRefreshRequest,
    },
};
use crate::{
    nns::proposals::report::{
        MAINNET_GOVERNANCE_CANISTER_ID, NNS_PROPOSAL_FETCHED_BY, NnsProposalHostError,
    },
    snapshot_cache::{SnapshotCompleteness, SnapshotJsonPaths, write_snapshot_json},
    subnet_catalog::{MAINNET_NETWORK, format_utc_timestamp_secs},
};

pub(super) fn publish_complete_nns_proposal_cache(
    request: &NnsProposalRefreshRequest,
    paths: &SnapshotJsonPaths,
    replaced_existing_cache: bool,
    complete: CompleteNnsProposalCollection,
) -> Result<NnsProposalRefreshReport, NnsProposalHostError> {
    let CompleteNnsProposalCollection {
        proposals,
        page_count,
        last_cursor,
    } = complete;
    let fetched_at = format_utc_timestamp_secs(request.now_unix_secs);
    let cache = NnsProposalCache {
        schema_version: NNS_PROPOSAL_CACHE_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: request.source_endpoint.clone(),
        fetched_at: fetched_at.clone(),
        fetched_by: NNS_PROPOSAL_FETCHED_BY.to_string(),
        domain: Some("nns".to_string()),
        entity: Some("governance".to_string()),
        collection: Some("proposals".to_string()),
        scope: Some("full".to_string()),
        metadata: NnsProposalCacheMetadata {
            governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        },
        completeness: SnapshotCompleteness::api_exhausted(
            request.page_size,
            page_count,
            proposals.len(),
            false,
        ),
        data: NnsProposalCacheRows { proposals },
    };
    let proposal_count = cache.data.proposals.len();
    write_snapshot_json(
        &paths.snapshot_path,
        &cache,
        |path, source| NnsProposalHostError::SerializeCache { path, source },
        NnsProposalHostError::Cache,
    )?;
    write_complete_attempt(
        &paths.refresh_attempt_path,
        request,
        NnsProposalAttemptProgress::new(page_count, proposal_count, last_cursor),
    )?;
    Ok(NnsProposalRefreshReport {
        schema_version: NNS_PROPOSAL_REFRESH_REPORT_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
        proposal_count,
        page_size: request.page_size,
        page_count,
        complete: true,
        replaced_existing_cache,
        wrote_cache: true,
        fetched_at,
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: NNS_PROPOSAL_FETCHED_BY.to_string(),
        cache_path: paths.snapshot_path.display().to_string(),
        refresh_attempt_path: paths.refresh_attempt_path.display().to_string(),
        refresh_lock_path: paths.refresh_lock_path.display().to_string(),
    })
}
