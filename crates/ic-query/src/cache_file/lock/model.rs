//! Inputs for confined descriptor-based refresh exclusion.

use std::path::Path;

///
/// RefreshLockRequest
///
/// Capability root and persistent coordination sidecar for one cache refresh.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RefreshLockRequest<'a> {
    /// Capability root that confines the sidecar.
    pub cache_root: &'a Path,
    /// Persistent empty regular sidecar, never unlinked during ordinary operation.
    pub lock_path: &'a Path,
}
