//! Module: sns::commands::run::common
//!
//! Responsibility: provide shared runtime helpers for SNS command execution.
//! Does not own: command specs, report building, or cache policy.
//! Boundary: adapts CLI clock and cache-root helpers into SNS command errors.

use crate::{cli::common::current_unix_secs, sns::commands::SnsCommandError, storage::cache_root};
use std::path::PathBuf;

pub(super) fn command_unix_secs() -> Result<u64, SnsCommandError> {
    Ok(current_unix_secs()?)
}

pub(super) fn command_cache_root() -> Result<PathBuf, SnsCommandError> {
    cache_root().map_err(|err| SnsCommandError::Usage(err.to_string()))
}
