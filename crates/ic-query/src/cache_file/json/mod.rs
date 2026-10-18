//! Module: cache_file::json
//!
//! Responsibility: shared JSON cache loading and validation.
//! Does not own: command-specific cache schemas, refresh execution, or process output.
//! Boundary: exposes schema and network validation helpers for cached reports.

mod load;
mod model;

pub use load::load_json_cache;
#[cfg(any(feature = "dashboard-host", feature = "nns-host", feature = "sns-host"))]
pub use model::CachedJsonReport;
pub use model::{JsonCacheReport, LoadJsonCacheRequest};
