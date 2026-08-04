//! Module: cache_file::lock::model
//!
//! Responsibility: refresh-lock request and file DTOs.
//! Does not own: lock acquisition, stale-lock policy, or guarded execution.
//! Boundary: defines the data exchanged by lock helpers.

use serde::{Deserialize, Serialize};
use std::path::Path;

pub(super) const REFRESH_LOCK_SCHEMA_VERSION: u32 = 1;

///
/// RefreshLockRequest
///
/// Inputs used to acquire a refresh lock for one cache target.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RefreshLockRequest<'a> {
    pub lock_path: &'a Path,
    pub target_path: &'a Path,
    pub network: &'a str,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
}

///
/// RefreshLockFile
///
/// Serialized lock file content used to detect active or stale refreshes.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RefreshLockFile {
    pub(super) schema_version: u32,
    pub(super) network: String,
    pub(super) pid: u32,
    pub(super) started_at_unix_ms: u64,
    pub(super) stale_after_seconds: u64,
    pub(super) target_path: String,
}

impl RefreshLockFile {
    pub(super) fn new(request: RefreshLockRequest<'_>, started_at_unix_ms: u64) -> Self {
        Self {
            schema_version: REFRESH_LOCK_SCHEMA_VERSION,
            network: request.network.to_string(),
            pid: std::process::id(),
            started_at_unix_ms,
            stale_after_seconds: request.lock_stale_after_seconds,
            target_path: request.target_path.display().to_string(),
        }
    }
}

///
/// RefreshLockEvidence
///
/// Strictly parsed refresh-lock identity, ownership, time, and stale policy.
///

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg(any(feature = "host", test))]
pub struct RefreshLockEvidence {
    pub(crate) schema_version: u32,
    pub(crate) network: String,
    pub(crate) pid: u32,
    pub(crate) started_at_unix_ms: u64,
    pub(crate) stale_after_seconds: u64,
    pub(crate) target_path: String,
}

#[cfg(any(feature = "host", test))]
impl From<RefreshLockFile> for RefreshLockEvidence {
    fn from(lock: RefreshLockFile) -> Self {
        Self {
            schema_version: lock.schema_version,
            network: lock.network,
            pid: lock.pid,
            started_at_unix_ms: lock.started_at_unix_ms,
            stale_after_seconds: lock.stale_after_seconds,
            target_path: lock.target_path,
        }
    }
}
