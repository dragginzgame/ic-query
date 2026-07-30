//! Module: cache_file::json::errors
//!
//! Responsibility: map generic JSON cache load failures to owner errors.
//! Does not own: cache-file IO or command-specific error enums.
//! Boundary: defines the error-mapping trait used by shared cache loaders.

use crate::HostCacheError;
use std::{io, path::PathBuf};

///
/// LoadJsonCacheErrorMapper
///
/// Maps shared JSON cache loading failures into command-family errors.
///

pub trait LoadJsonCacheErrorMapper {
    type Error;

    fn missing_cache(&self, path: PathBuf) -> Self::Error;
    fn read_cache(&self, path: PathBuf, source: io::Error) -> Self::Error;
    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error;
    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error;
    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error;
}

///
/// HostJsonCacheErrorMapper
///
/// Maps generic JSON cache failures to the shared component-labelled host error.
///

pub struct HostJsonCacheErrorMapper {
    component: &'static str,
}

impl HostJsonCacheErrorMapper {
    pub const fn new(component: &'static str) -> Self {
        Self { component }
    }
}

impl LoadJsonCacheErrorMapper for HostJsonCacheErrorMapper {
    type Error = HostCacheError;

    fn missing_cache(&self, path: PathBuf) -> Self::Error {
        HostCacheError::missing_cache(self.component, path)
    }

    fn read_cache(&self, path: PathBuf, source: io::Error) -> Self::Error {
        HostCacheError::read_cache(self.component, path, source)
    }

    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error {
        HostCacheError::parse_cache(self.component, path, source)
    }

    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error {
        HostCacheError::unsupported_cache_schema_version(self.component, version, expected)
    }

    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error {
        HostCacheError::network_mismatch(self.component, requested, actual)
    }
}
