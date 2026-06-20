//! Module: nns::proposals::report::cache
//!
//! Responsibility: complete NNS proposal snapshot refresh and inspection.
//! Does not own: command parsing, live governance transport, or text rendering.
//! Boundary: stores complete proposal snapshots and refresh-attempt metadata.

mod attempt;
mod model;
mod paths;
mod refresh;
mod reports;

use crate::{cache_file::CacheFileError, nns::proposals::report::NnsProposalHostError};

pub(in crate::nns) use model::{
    NnsProposalCacheListReport, NnsProposalCacheListRequest, NnsProposalCacheStatusReport,
    NnsProposalCacheStatusRequest, NnsProposalRefreshAttemptStatus, NnsProposalRefreshReport,
    NnsProposalRefreshRequest,
};
pub(in crate::nns::proposals) use refresh::refresh_nns_proposal_cache;
pub(in crate::nns::proposals) use reports::{
    build_nns_proposal_cache_list_report, build_nns_proposal_cache_status_report,
};

#[cfg(test)]
mod tests;

const NNS_PROPOSAL_CACHE_SCHEMA_VERSION: u32 = 1;
const NNS_PROPOSAL_CACHE_LIST_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_PROPOSAL_CACHE_STATUS_REPORT_SCHEMA_VERSION: u32 = 1;
const NNS_PROPOSAL_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;

fn cache_file_error(err: CacheFileError) -> NnsProposalHostError {
    NnsProposalHostError::Cache(match err {
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
