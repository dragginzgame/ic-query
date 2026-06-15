use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RefreshLockRequest<'a> {
    pub lock_path: &'a Path,
    pub target_path: &'a Path,
    pub network: &'a str,
    pub now_unix_secs: u64,
    pub lock_stale_after_seconds: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(super) struct RefreshLockFile {
    pub(super) schema_version: u32,
    pub(super) network: String,
    pub(super) pid: u32,
    pub(super) started_at_unix_ms: u64,
    #[serde(alias = "catalog_path", alias = "cache_path")]
    pub(super) target_path: String,
}

impl RefreshLockFile {
    pub(super) fn new(request: RefreshLockRequest<'_>, started_at_unix_ms: u64) -> Self {
        Self {
            schema_version: 1,
            network: request.network.to_string(),
            pid: std::process::id(),
            started_at_unix_ms,
            target_path: request.target_path.display().to_string(),
        }
    }
}
