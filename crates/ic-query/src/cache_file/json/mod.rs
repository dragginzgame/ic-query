//! Module: cache_file::json
//!
//! Responsibility: shared JSON cache loading and validation.
//! Does not own: command-specific cache schemas, refresh execution, or process output.
//! Boundary: exposes schema and network validation helpers for cached reports.

mod errors;
mod load;
mod model;

pub use errors::HostJsonCacheErrorMapper;
pub use errors::LoadJsonCacheErrorMapper;
pub use errors::OwnerJsonCacheErrorMapper;
pub use load::{load_json_cache, load_json_cache_strict};
pub use model::{CachedJsonReport, JsonCacheReport, LoadJsonCacheRequest};
