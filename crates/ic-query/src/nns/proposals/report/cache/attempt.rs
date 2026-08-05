//! Module: nns::proposals::report::cache::attempt
//!
//! Responsibility: read and write NNS proposal refresh-attempt metadata.
//! Does not own: live proposal paging, cache publication, or text rendering.
//! Boundary: persists refresh lifecycle status for cache status reports.

use super::NNS_PROPOSAL_CACHE_COMPONENT;
use crate::{
    cache::CacheRefreshAttemptStatus,
    nns::{
        NnsGovernanceRefreshAttemptStatus, NnsGovernanceRefreshRequest,
        governance::{
            NnsGovernanceAttemptReadError, read_governance_refresh_attempt_status,
            write_failed_governance_refresh_attempt, write_governance_refresh_attempt,
        },
        proposals::report::NnsProposalHostError,
    },
    snapshot_cache::SnapshotRefreshProgress,
    subnet_catalog::MAINNET_NETWORK,
};
use std::path::Path;

pub(super) fn read_attempt_status(
    cache_root: &Path,
    path: &Path,
) -> Option<NnsGovernanceRefreshAttemptStatus> {
    read_governance_refresh_attempt_status(
        cache_root,
        path,
        MAINNET_NETWORK,
        NNS_PROPOSAL_CACHE_COMPONENT,
    )
    .ok()
    .flatten()
}

pub(super) fn read_attempt_status_strict(
    cache_root: &Path,
    path: &Path,
    expected_network: &str,
) -> Result<Option<NnsGovernanceRefreshAttemptStatus>, NnsProposalHostError> {
    read_governance_refresh_attempt_status(
        cache_root,
        path,
        expected_network,
        NNS_PROPOSAL_CACHE_COMPONENT,
    )
    .map_err(map_attempt_read_error)
}

pub(super) fn write_starting_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
) -> Result<(), NnsProposalHostError> {
    write_attempt_status(
        path,
        request,
        CacheRefreshAttemptStatus::Running,
        SnapshotRefreshProgress::default(),
        None,
    )
}

pub(super) fn write_running_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    progress: SnapshotRefreshProgress,
) -> Result<(), NnsProposalHostError> {
    write_attempt_status(
        path,
        request,
        CacheRefreshAttemptStatus::Running,
        progress,
        None,
    )
}

pub(super) fn write_complete_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    progress: SnapshotRefreshProgress,
) -> Result<(), NnsProposalHostError> {
    write_attempt_status(
        path,
        request,
        CacheRefreshAttemptStatus::Complete,
        progress,
        None,
    )
}

pub(super) fn write_failed_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    err: &NnsProposalHostError,
) {
    let _ = write_failed_governance_refresh_attempt(
        path,
        request,
        NNS_PROPOSAL_CACHE_COMPONENT,
        err.to_string(),
    );
}

fn map_attempt_read_error(error: NnsGovernanceAttemptReadError) -> NnsProposalHostError {
    match error {
        NnsGovernanceAttemptReadError::Cache(error) => NnsProposalHostError::Cache(error),
        NnsGovernanceAttemptReadError::Invalid { path, reason } => {
            NnsProposalHostError::InvalidRefreshAttempt { path, reason }
        }
    }
}

fn write_attempt_status(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    status: CacheRefreshAttemptStatus,
    progress: SnapshotRefreshProgress,
    last_error: Option<String>,
) -> Result<(), NnsProposalHostError> {
    write_governance_refresh_attempt(
        path,
        request,
        NNS_PROPOSAL_CACHE_COMPONENT,
        status,
        progress,
        last_error,
    )
    .map_err(NnsProposalHostError::Cache)
}
