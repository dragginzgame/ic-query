//! Module: cache_file::json::model
//!
//! Responsibility: shared JSON cache report contracts.
//! Does not own: filesystem IO, refresh policy, or command-specific report fields.
//! Boundary: defines minimal metadata needed for schema and network validation.

use std::path::{Path, PathBuf};

///
/// CachedJsonReport
///
/// Loaded JSON cache report paired with the file path it came from.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedJsonReport<T> {
    /// Managed file containing the report.
    pub path: PathBuf,
    /// Validated schema and network report.
    pub report: T,
}

///
/// JsonCacheReport
///
/// Minimal metadata every JSON cache report exposes for validation.
///

pub trait JsonCacheReport {
    /// Persisted schema identifier.
    fn schema_version(&self) -> u32;
    /// Network identity represented by the cache.
    fn network(&self) -> &str;
}

///
/// LoadJsonCacheRequest
///
/// Inputs needed to load and validate one JSON cache report.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadJsonCacheRequest<'a> {
    /// Capability root that confines the managed cache path.
    pub cache_root: &'a Path,
    /// Managed file containing the report.
    pub path: PathBuf,
    /// Required network identity.
    pub network: &'a str,
    /// Supported persisted schema identifier.
    pub expected_schema_version: u32,
    /// Maximum admitted serialized bytes, selected by the cache owner.
    pub maximum_bytes: u64,
}
