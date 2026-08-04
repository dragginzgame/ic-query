//! Shared cache report models and host-only local inventory helpers.

mod completeness;
mod model;
#[cfg(feature = "host")]
mod status;
#[cfg(feature = "host")]
mod text;

pub use completeness::{CacheCollectionCompleteness, validate_cache_collection_completeness};
pub use model::{
    CACHE_STATUS_REPORT_SCHEMA_VERSION, CacheAgeStatus, CacheHeaderStatus, CacheRecoveryPolicy,
    CacheRefreshAttemptStatus, CacheRefreshLockStatus, CacheRefreshLockStatusRow,
    CacheStatusReport, CacheStatusRequest, CacheStatusRow, CacheValidationStatus,
};
#[cfg(feature = "host")]
pub use status::{CacheStatusError, build_cache_status_report};
#[cfg(feature = "host")]
pub use text::cache_status_report_text;
