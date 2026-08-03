//! Module: nns::neuron::report::cache::attempt
//!
//! Responsibility: read and write NNS neuron refresh-attempt metadata.
//! Does not own: live neuron paging, cache publication, or text rendering.
//! Boundary: persists refresh lifecycle status for cache status reports.

use super::NNS_NEURON_CACHE_COMPONENT;
use crate::{
    cache::CacheRefreshAttemptStatus,
    nns::{
        NnsGovernanceRefreshAttemptStatus, NnsGovernanceRefreshRequest,
        governance::{
            NnsGovernanceAttemptReadError, read_governance_refresh_attempt_status,
            write_failed_governance_refresh_attempt, write_governance_refresh_attempt,
        },
        neuron::report::NnsNeuronHostError,
    },
    snapshot_cache::SnapshotRefreshProgress,
};
use std::path::Path;

pub(super) fn write_starting_attempt(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
) -> Result<(), NnsNeuronHostError> {
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
) -> Result<(), NnsNeuronHostError> {
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
) -> Result<(), NnsNeuronHostError> {
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
    error: &NnsNeuronHostError,
) -> Result<(), NnsNeuronHostError> {
    write_failed_governance_refresh_attempt(
        path,
        request,
        NNS_NEURON_CACHE_COMPONENT,
        error.to_string(),
    )
    .map_err(NnsNeuronHostError::Cache)
}

pub(super) fn read_attempt_status(
    path: &Path,
    network: &str,
) -> Result<Option<NnsGovernanceRefreshAttemptStatus>, NnsNeuronHostError> {
    read_governance_refresh_attempt_status(path, network, NNS_NEURON_CACHE_COMPONENT)
        .map_err(map_attempt_read_error)
}

fn write_attempt_status(
    path: &Path,
    request: &NnsGovernanceRefreshRequest,
    status: CacheRefreshAttemptStatus,
    progress: SnapshotRefreshProgress,
    last_error: Option<String>,
) -> Result<(), NnsNeuronHostError> {
    write_governance_refresh_attempt(
        path,
        request,
        NNS_NEURON_CACHE_COMPONENT,
        status,
        progress,
        last_error,
    )
    .map_err(NnsNeuronHostError::Cache)
}

fn map_attempt_read_error(error: NnsGovernanceAttemptReadError) -> NnsNeuronHostError {
    match error {
        NnsGovernanceAttemptReadError::Cache(error) => NnsNeuronHostError::Cache(error),
        NnsGovernanceAttemptReadError::Invalid { path, reason } => {
            NnsNeuronHostError::InvalidCache { path, reason }
        }
    }
}
