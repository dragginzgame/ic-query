//! Module: cache_file::lock
//!
//! Responsibility: shared refresh-lock acquisition and cleanup.
//! Does not own: cache refresh work or cache report serialization.
//! Boundary: exposes lock requests and guarded execution helpers.

mod acquire;
mod guard;
mod model;
mod run;
#[cfg(test)]
mod tests;

#[cfg(feature = "host")]
pub use acquire::inspect_refresh_lock;
#[cfg(feature = "host")]
pub use model::RefreshLockEvidence;
pub use model::RefreshLockRequest;
#[cfg(feature = "nns-topology-host")]
pub use run::with_refresh_lock;
pub use run::with_refresh_lock_async;
