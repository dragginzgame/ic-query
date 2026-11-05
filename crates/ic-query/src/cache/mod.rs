//! Shared complete-cache and refresh-attempt contracts.

mod completeness;
mod model;

pub use completeness::{CacheCollectionCompleteness, validate_cache_collection_completeness};
pub use model::{CacheRefreshAttemptStatus, CacheValidationStatus, SnapshotRefreshAttemptStatus};
