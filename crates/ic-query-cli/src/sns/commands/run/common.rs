//! Module: sns::commands::run::common
//!
//! Responsibility: provide shared runtime helpers for SNS command execution.
//! Does not own: command specs, report building, or cache policy.
//! Boundary: maps cache-root failures into SNS command errors.

use crate::{sns::commands::SnsCommandError, storage::cache_root};
use std::path::PathBuf;

pub(super) fn command_cache_root() -> Result<PathBuf, SnsCommandError> {
    cache_root().map_err(|err| SnsCommandError::Usage(err.to_string()))
}
