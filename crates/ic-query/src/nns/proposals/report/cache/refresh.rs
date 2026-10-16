//! Collect complete Governance snapshots through family-specific paging.

use super::{
    NNS_PROPOSAL_CACHE_COMPONENT,
    collection::fetch_complete_nns_proposal_collection,
    model::{CompleteNnsProposalCollection, NnsProposalCacheRows},
    paths::nns_proposal_cache_paths,
};
use crate::{
    QueryProgress,
    nns::{
        LiveNnsSource, NnsGovernanceRefreshRequest,
        governance::{NnsGovernanceRefreshReport, run_governance_snapshot_refresh},
        proposals::report::{
            NNS_PROPOSAL_FETCHED_BY, NnsProposalHostError, enforce_mainnet_network,
            source::{NnsProposalSource, validate_proposal_page_size},
        },
    },
    progress::IgnoreQueryProgress,
    snapshot_cache::SnapshotRefreshProgress,
};

/// Default stale-lock threshold in seconds for proposal refreshes.
pub const DEFAULT_NNS_PROPOSAL_REFRESH_LOCK_STALE_SECONDS: u64 = 30 * 60;

/// Refresh a complete NNS proposal snapshot using the live NNS proposal source.
pub async fn refresh_nns_proposal_cache(
    request: &NnsGovernanceRefreshRequest,
) -> Result<NnsGovernanceRefreshReport, NnsProposalHostError> {
    refresh_nns_proposal_cache_with_source(request, &LiveNnsSource::default()).await
}

/// Refresh a complete NNS proposal snapshot and emit structured progress events.
pub async fn refresh_nns_proposal_cache_with_progress(
    request: &NnsGovernanceRefreshRequest,
    progress: &mut dyn QueryProgress,
) -> Result<NnsGovernanceRefreshReport, NnsProposalHostError> {
    refresh_nns_proposal_cache_with_source_and_progress(
        request,
        &LiveNnsSource::default(),
        progress,
    )
    .await
}

/// Collect proposals through the supplied source and atomically publish a complete snapshot.
pub async fn refresh_nns_proposal_cache_with_source(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsProposalSource,
) -> Result<NnsGovernanceRefreshReport, NnsProposalHostError> {
    let mut progress = IgnoreQueryProgress;
    refresh_nns_proposal_cache_with_source_and_progress(request, source, &mut progress).await
}

pub(super) async fn refresh_nns_proposal_cache_with_source_and_progress(
    request: &NnsGovernanceRefreshRequest,
    source: &dyn NnsProposalSource,
    progress: &mut dyn QueryProgress,
) -> Result<NnsGovernanceRefreshReport, NnsProposalHostError> {
    validate_proposal_page_size(request.page_size)?;
    enforce_mainnet_network(&request.network)?;
    let paths = nns_proposal_cache_paths(&request.cache_root, &request.network);
    run_governance_snapshot_refresh(
        request,
        &paths,
        NNS_PROPOSAL_CACHE_COMPONENT,
        DEFAULT_NNS_PROPOSAL_REFRESH_LOCK_STALE_SECONDS,
        async |context| {
            let CompleteNnsProposalCollection { proposals, state } =
                fetch_complete_nns_proposal_collection(
                    request,
                    source,
                    &paths.refresh_attempt_path,
                    progress,
                )
                .await?;
            let publication_progress = SnapshotRefreshProgress::new(
                state.pages_fetched(),
                proposals.len(),
                state
                    .next_before_proposal_id()
                    .map(|cursor| cursor.to_string()),
            );
            context.publish(
                "proposals",
                NNS_PROPOSAL_FETCHED_BY,
                publication_progress,
                NnsProposalCacheRows {
                    collection_state: state,
                    proposals,
                },
            )
        },
    )
    .await
}
