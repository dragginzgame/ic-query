//! Module: snapshot_cache
//!
//! Responsibility: shared complete-snapshot cache primitives.
//! Does not own: NNS/SNS cache schemas, command parsing, or text rendering.
//! Boundary: provides snapshot keys, paths, envelopes, locks, attempts, and paged refresh flow.

mod attempt;
mod json;
mod key;
mod lifecycle;
mod model;
mod paged;
mod paths;
mod refresh;

pub use attempt::{
    SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION, SnapshotRefreshAttempt, read_snapshot_refresh_attempt,
    write_snapshot_refresh_attempt,
};
pub use json::{load_complete_snapshot_for_key, load_snapshot_header, write_snapshot_json};
pub use key::SnapshotKey;
pub use lifecycle::{
    LockedSnapshotRefreshRequest, run_snapshot_refresh_with_attempts, with_locked_snapshot_refresh,
};
pub use model::{
    SNAPSHOT_CACHE_STATUS_INVALID, SNAPSHOT_CACHE_STATUS_OK, SnapshotCompleteness,
    SnapshotEnvelope, SnapshotHeader, SnapshotIdentityMismatch, SnapshotReport,
};
pub use paged::{PagedCollectionPage, PagedCollectionState};
pub use paths::{SnapshotJsonPaths, collect_full_collection_snapshot_paths, snapshot_network_dir};
pub use refresh::{PagedSnapshotRefresh, run_paged_snapshot_refresh};

#[cfg(test)]
mod tests;
