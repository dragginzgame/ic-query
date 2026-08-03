//! Module: sns::report::proposals_cache::refresh::publish
//!
//! Responsibility: publish complete SNS proposal snapshots.
//! Does not own: refresh locking, live proposal paging, or command parsing.
//! Boundary: writes complete cache JSON and complete-attempt metadata atomically.

use super::SnsProposalsRefreshContext;
use crate::sns::report::{
    SnsHostError, SnsProposalsRefreshReport,
    cache_refresh::publish_complete_sns_snapshot,
    proposals_cache::{
        SNS_PROPOSALS_CACHE_SCHEMA_VERSION, SNS_PROPOSALS_REFRESH_REPORT_SCHEMA_VERSION,
        model::{CompleteSnsProposals, SnsProposalsCacheRows},
    },
};

pub(super) fn publish_complete_sns_proposals_cache(
    context: &SnsProposalsRefreshContext<'_>,
    complete: CompleteSnsProposals,
) -> Result<SnsProposalsRefreshReport, SnsHostError> {
    let page_count = complete.page_count;
    let proposal_count = complete.rows.len();
    let attempt_finalization_error = publish_complete_sns_snapshot(
        context,
        SNS_PROPOSALS_CACHE_SCHEMA_VERSION,
        page_count,
        proposal_count,
        complete.last_cursor,
        SnsProposalsCacheRows {
            proposals: complete.rows,
        },
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
        attempt_finalization_error,
    })
}
