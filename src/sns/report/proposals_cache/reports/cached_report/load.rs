//! Module: sns::report::proposals_cache::reports::cached_report::load
//!
//! Responsibility: load or auto-refresh complete SNS proposal caches.
//! Does not own: report projection, local view filtering, or refresh internals.
//! Boundary: applies shared missing-cache policy for cache-compatible proposal views.

use super::super::super::{
    SNS_PROPOSALS_AUTO_REFRESH_PAGE_SIZE, model::SnsProposalsCache,
    refresh_sns_proposals_cache_with_source, storage::load_sns_proposals_cache_for_input,
};
use crate::{
    cache_file::load_or_refresh_missing_cache,
    sns::report::{
        SnsHostError, SnsProposalsRefreshRequest, SnsProposalsRequest, source::SnsProposalsSource,
    },
};
use std::path::Path;

pub(super) fn load_or_refresh_sns_proposals_cache(
    request: &SnsProposalsRequest,
    icp_root: &Path,
    source: &dyn SnsProposalsSource,
) -> Result<SnsProposalsCache, SnsHostError> {
    load_or_refresh_missing_cache(
        "SNS proposals",
        &request.source_endpoint,
        || load_sns_proposals_cache_for_input(icp_root, &request.network, &request.input),
        || {
            refresh_sns_proposals_cache_with_source(
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
            )
            .map(|_| ())
        },
        |err| match err {
            SnsHostError::MissingProposalsCache { path } => Ok(path),
            err => Err(err),
        },
    )
}
