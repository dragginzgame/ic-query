//! Module: snapshot_cache
//!
//! Responsibility: shared complete-snapshot cache primitives.
//! Does not own: NNS/SNS cache schemas, command parsing, or text rendering.
//! Boundary: provides snapshot keys, paths, envelopes, locks, attempts, and paged refresh flow.

#[cfg(feature = "host")]
mod attempt;
mod json;
mod key;
mod lifecycle;
mod model;
#[cfg(feature = "host")]
mod paged;
mod paths;
#[cfg(feature = "host")]
mod refresh;

#[cfg(feature = "host")]
pub use attempt::{
    SNAPSHOT_REFRESH_ATTEMPT_SCHEMA_VERSION, SnapshotRefreshAttempt,
    SnapshotRefreshAttemptReadError, SnapshotRefreshProgress, current_attempt_timestamp,
    read_snapshot_refresh_attempt_strict, validate_snapshot_refresh_attempt,
    write_snapshot_refresh_attempt,
};
#[cfg(feature = "host")]
pub use json::load_snapshot_header;
pub use json::{load_complete_snapshot_for_key, write_snapshot_json};
pub use key::SnapshotKey;
pub use lifecycle::{LockedSnapshotRefreshRequest, with_locked_snapshot_refresh};
#[cfg(feature = "host")]
pub use lifecycle::{publish_snapshot_with_attempt, run_snapshot_refresh_with_attempts};
#[cfg(feature = "host")]
pub use model::SnapshotHeader;
pub use model::{SnapshotEnvelope, SnapshotIdentityMismatch, SnapshotReport};
#[cfg(feature = "host")]
pub use paged::PagedCollectionPage;
#[cfg(feature = "host")]
pub use paged::{CompletePagedCollection, PagedCollectionState};
pub use paths::SnapshotJsonPaths;
#[cfg(feature = "host")]
pub use paths::{
    collect_full_collection_attempt_paths, collect_full_collection_snapshot_paths,
    snapshot_network_dir,
};
#[cfg(feature = "host")]
pub use refresh::{PagedSnapshotRefresh, run_paged_snapshot_refresh_with_progress};

#[cfg(all(test, feature = "host"))]
mod tests;
