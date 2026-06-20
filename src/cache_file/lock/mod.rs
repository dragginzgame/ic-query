//! Module: cache_file::lock
//!
//! Responsibility: shared refresh-lock acquisition and cleanup.
//! Does not own: cache refresh work or cache report serialization.
//! Boundary: exposes lock requests and guarded execution helpers.

mod acquire;
mod guard;
mod model;
mod run;

pub use model::RefreshLockRequest;
pub use run::with_refresh_lock;
