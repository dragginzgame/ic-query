//! Shared cache report models and host-only local inventory helpers.

mod model;
#[cfg(feature = "host")]
mod status;
#[cfg(feature = "host")]
mod text;

pub use model::{
    CACHE_STATUS_REPORT_SCHEMA_VERSION, CacheFileStatus, CacheRefreshLockStatus,
    CacheRefreshLockStatusRow, CacheStatusReport, CacheStatusRequest, CacheStatusRow,
    CacheValidationStatus,
};
#[cfg(feature = "host")]
pub use status::{CacheStatusError, build_cache_status_report};
#[cfg(feature = "host")]
pub use text::cache_status_report_text;
