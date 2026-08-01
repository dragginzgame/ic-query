//! Module: cache_file::write
//!
//! Responsibility: shared atomic and refresh-cache write helpers.
//! Does not own: JSON cache schemas, refresh locking internals, or report construction.
//! Boundary: exposes parent creation, atomic text writes, and refresh publication helpers.

mod atomic;
mod output;
mod path;
#[cfg(feature = "host")]
mod refresh;

pub use atomic::write_text_atomically;
pub use output::write_text_output;
pub use path::create_parent_directory;
#[cfg(feature = "host")]
pub use refresh::{RefreshCacheWriteRequest, RefreshCacheWriteResult, write_json_refresh_cache};

#[cfg(all(test, feature = "host"))]
mod tests;
