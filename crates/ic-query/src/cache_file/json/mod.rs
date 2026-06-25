//! Module: cache_file::json
//!
//! Responsibility: shared JSON cache loading and refresh announcements.
//! Does not own: command-specific cache schemas or refresh execution.
//! Boundary: exposes schema/network validation helpers and user-facing refresh notices.

mod announce;
mod errors;
mod load;
mod model;

pub use announce::announce_cache_refresh;
pub use errors::LoadJsonCacheErrorMapper;
pub use load::load_json_cache;
pub use model::{CachedJsonReport, JsonCacheReport, LoadJsonCacheRequest};
