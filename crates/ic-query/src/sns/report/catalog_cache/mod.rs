//! Module: sns::report::catalog_cache
//!
//! Responsibility: persist and project the joined deployed-SNS catalog.
//! Does not own: live transport, command parsing, or generic cache primitives.
//! Boundary: refreshes the all-SNS metadata fan-out only for catalog operations.

mod model;
mod run;
mod text;

pub use model::{SnsCatalogCacheRequest, SnsCatalogRefreshReport, SnsCatalogRefreshRequest};
pub use run::{
    DEFAULT_SNS_CATALOG_REFRESH_LOCK_STALE_SECONDS, DEFAULT_SNS_CATALOG_STALE_AFTER_SECONDS,
    build_sns_list_report_from_cache, build_sns_list_report_from_cache_or_refresh,
    build_sns_list_report_from_cache_or_refresh_with_source, refresh_sns_catalog,
    refresh_sns_catalog_with_source, sns_catalog_cache_path, sns_catalog_refresh_lock_path,
};
pub use text::sns_catalog_refresh_report_text;

const SNS_CATALOG_CACHE_SCHEMA_VERSION: u32 = 1;
const SNS_CATALOG_REFRESH_REPORT_SCHEMA_VERSION: u32 = 1;
