//! Module: sns::commands::run::common
//!
//! Responsibility: provide shared runtime helpers for SNS command execution.
//! Does not own: command specs, report building, or cache policy.
//! Boundary: adapts CLI-level helpers into SNS command errors and request parts.

use crate::{
    cli::common::{OutputFormat, current_unix_secs},
    sns::commands::{SnsCommandError, options::SnsLookupOptions},
    storage::cache_root,
};
use std::path::PathBuf;

///
/// SnsLookupCommandParts
///
/// Runtime inputs shared by SNS commands that resolve an SNS selector.
///

pub(super) struct SnsLookupCommandParts {
    pub(super) format: OutputFormat,
    pub(super) network: String,
    pub(super) source_endpoint: String,
    pub(super) now_unix_secs: u64,
    pub(super) input: String,
}

///
/// SnsCachedLookupCommandParts
///
/// Runtime inputs for SNS selector commands that also read local cache state.
///

pub(in crate::sns::commands::run) struct SnsCachedLookupCommandParts {
    pub(in crate::sns::commands::run) format: OutputFormat,
    pub(in crate::sns::commands::run) network: String,
    pub(in crate::sns::commands::run) source_endpoint: String,
    pub(in crate::sns::commands::run) now_unix_secs: u64,
    pub(in crate::sns::commands::run) input: String,
    pub(in crate::sns::commands::run) cache_root: PathBuf,
}

///
/// SnsCacheCommandParts
///
/// Runtime inputs shared by SNS cache inspection commands.
///

pub(in crate::sns::commands::run) struct SnsCacheCommandParts {
    pub(in crate::sns::commands::run) format: OutputFormat,
    pub(in crate::sns::commands::run) network: String,
    pub(in crate::sns::commands::run) cache_root: PathBuf,
}

pub(super) fn command_unix_secs() -> Result<u64, SnsCommandError> {
    Ok(current_unix_secs()?)
}

pub(super) fn command_cache_root() -> Result<PathBuf, SnsCommandError> {
    cache_root().map_err(|err| SnsCommandError::Usage(err.to_string()))
}

pub(super) fn lookup_command_parts(
    options: SnsLookupOptions,
) -> Result<SnsLookupCommandParts, SnsCommandError> {
    Ok(SnsLookupCommandParts {
        format: options.format,
        network: options.network,
        source_endpoint: options.source_endpoint,
        now_unix_secs: command_unix_secs()?,
        input: options.input,
    })
}

pub(in crate::sns::commands::run) fn cached_lookup_command_parts(
    options: SnsLookupOptions,
) -> Result<SnsCachedLookupCommandParts, SnsCommandError> {
    let parts = lookup_command_parts(options)?;
    Ok(SnsCachedLookupCommandParts {
        format: parts.format,
        network: parts.network,
        source_endpoint: parts.source_endpoint,
        now_unix_secs: parts.now_unix_secs,
        input: parts.input,
        cache_root: command_cache_root()?,
    })
}

pub(in crate::sns::commands::run) fn cache_command_parts(
    format: OutputFormat,
    network: String,
) -> Result<SnsCacheCommandParts, SnsCommandError> {
    Ok(SnsCacheCommandParts {
        format,
        network,
        cache_root: command_cache_root()?,
    })
}
