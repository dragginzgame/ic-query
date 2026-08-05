//! Module: snapshot_cache
//!
//! Responsibility: shared complete-snapshot cache primitives.
//! Does not own: NNS/SNS cache schemas, command parsing, or text rendering.
//! Boundary: provides snapshot keys, paths, envelopes, locks, attempts, and paged refresh flow.

#[cfg(any(feature = "icrc-host", feature = "sns-host"))]
mod attempt;
mod json;
mod key;
mod lifecycle;
#[cfg(any(feature = "dashboard-host", feature = "sns-host"))]
mod model;
#[cfg(feature = "sns-host")]
mod paged;
mod paths;
#[cfg(feature = "sns-host")]
mod refresh;

#[cfg(any(feature = "icrc-host", feature = "sns-host"))]
pub use attempt::{
    SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION, SnapshotRefreshAttempt,
    SnapshotRefreshAttemptReadError, SnapshotRefreshProgress, current_attempt_timestamp,
    read_snapshot_refresh_attempt_strict, validate_snapshot_refresh_attempt,
    write_snapshot_refresh_attempt,
};
#[cfg(any(feature = "dashboard-host", feature = "sns-host"))]
pub use json::load_complete_snapshot_for_key;
#[cfg(feature = "sns-host")]
pub use json::load_snapshot_header;
pub use json::write_snapshot_json;
pub use key::SnapshotKey;
pub use lifecycle::{LockedSnapshotRefreshRequest, with_locked_snapshot_refresh};
#[cfg(any(feature = "icrc-host", feature = "sns-host"))]
pub use lifecycle::{publish_snapshot_with_attempt, run_snapshot_refresh_with_attempts};
#[cfg(feature = "sns-host")]
pub use model::SnapshotHeader;
#[cfg(any(feature = "dashboard-host", feature = "sns-host"))]
pub use model::{SnapshotEnvelope, SnapshotIdentityMismatch, SnapshotReport};
#[cfg(feature = "sns-host")]
pub use paged::PagedCollectionPage;
#[cfg(feature = "sns-host")]
pub use paged::{CompletePagedCollection, PagedCollectionState};
pub use paths::SnapshotJsonPaths;
#[cfg(feature = "sns-host")]
pub use paths::{
    collect_full_collection_attempt_paths, collect_full_collection_snapshot_paths,
    snapshot_network_dir,
};
#[cfg(feature = "sns-host")]
pub use refresh::{PagedSnapshotRefresh, run_paged_snapshot_refresh_with_progress};

#[cfg(all(test, feature = "host"))]
mod tests;
