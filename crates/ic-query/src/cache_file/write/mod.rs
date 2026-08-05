//! Module: cache_file::write
//!
//! Responsibility: shared atomic and refresh-cache write helpers.
//! Does not own: JSON cache schemas, refresh locking internals, or report construction.
//! Boundary: exposes parent creation, atomic text writes, and refresh publication helpers.

#[cfg(any(feature = "host", feature = "subnet-catalog-host"))]
mod output;
#[cfg(any(feature = "host", feature = "subnet-catalog-host"))]
mod path;
#[cfg(feature = "host")]
mod refresh;

#[cfg(any(feature = "host", feature = "subnet-catalog-host"))]
pub use output::write_text_output;
#[cfg(feature = "host")]
pub use refresh::{RefreshCacheWriteRequest, RefreshCacheWriteResult, write_json_refresh_cache};
