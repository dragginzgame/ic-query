//! Module: sns::report::proposals_cache::refresh::publish
//!
//! Responsibility: publish complete SNS proposal snapshots.
//! Does not own: refresh locking, live proposal paging, or command parsing.
//! Boundary: writes complete cache JSON and complete-attempt metadata atomically.

use super::context::SnsProposalsRefreshContext;
use crate::{
    snapshot_cache::{SnapshotCompleteness, write_snapshot_json},
    sns::report::{
        SnsHostError, SnsProposalRow, SnsProposalsRefreshReport,
        proposals_cache::{
            SNS_PROPOSALS_CACHE_SCHEMA_VERSION, SNS_PROPOSALS_REFRESH_REPORT_SCHEMA_VERSION,
            attempt::{SnsProposalsAttemptProgress, write_complete_attempt},
            model::{
                CompleteSnsProposals, SnsProposalsCache, SnsProposalsCacheMetadata,
                SnsProposalsCacheRows,
            },
        },
        source::{MainnetSns, MainnetSnsList},
    },
};

pub(super) fn publish_complete_sns_proposals_cache(
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
        SnsHostError::Cache,
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
        domain: Some("sns".to_string()),
        entity: Some(sns.root_canister_id.clone()),
        collection: Some("proposals".to_string()),
        scope: Some("full".to_string()),
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
