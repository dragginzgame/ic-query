//! User-level cache inventory reports.

mod model;
mod status;
mod text;

pub use model::{
    CACHE_STATUS_REPORT_SCHEMA_VERSION, CacheStatusReport, CacheStatusRequest, CacheStatusRow,
};
pub use status::{CacheStatusError, build_cache_status_report};
pub use text::cache_status_report_text;
