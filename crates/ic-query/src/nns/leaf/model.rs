//! Module: nns::leaf::model
//!
//! Responsibility: shared cache request contracts for generic NNS leaf reports.
//! Does not own: command construction, report rendering, or cache file IO.
//! Boundary: defines the cache traits shared by data-center, node, operator, and provider reports.

use std::path::Path;

///
/// NnsLeafCacheRequest
///
/// Cache identity contract shared by generic NNS leaf report requests.
///

pub(in crate::nns) trait NnsLeafCacheRequest: Clone {
    fn from_root_network(icp_root: &Path, network: &str) -> Self;
    fn icp_root(&self) -> &Path;
    fn network(&self) -> &str;
}

///
/// NnsLeafRefreshRequest
///
/// Report-builder request contract for generic NNS leaf refresh operations.
///

pub(in crate::nns) trait NnsLeafRefreshRequest {
    type Cache: NnsLeafCacheRequest;

    fn cache(&self) -> &Self::Cache;
    fn now_unix_secs(&self) -> u64;
    fn lock_stale_after_seconds(&self) -> u64;
    fn dry_run(&self) -> bool;
    fn output_path(&self) -> Option<&Path>;
}
