//! Module: cache_file::json::errors
//!
//! Responsibility: map generic JSON cache load failures to owner errors.
//! Does not own: cache-file IO or command-specific error enums.
//! Boundary: defines the error-mapping trait used by shared cache loaders.

use crate::{CacheFileError, HostCacheError};
use std::path::PathBuf;

///
/// LoadJsonCacheErrorMapper
///
/// Maps shared JSON cache loading failures into command-family errors.
///

pub trait LoadJsonCacheErrorMapper {
    type Error;

    fn missing_cache(&self, path: PathBuf) -> Self::Error;
    /// Map a capability-rooted cache operation failure.
    fn cache_operation(&self, source: CacheFileError) -> Self::Error;
    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error;
    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error;
    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error;
}

///
/// HostJsonCacheErrorMapper
///
/// Maps generic JSON cache failures to the shared component-labelled host error.
///

#[cfg(any(feature = "icrc-host", feature = "nns-topology-host"))]
pub struct HostJsonCacheErrorMapper {
    component: &'static str,
}

#[cfg(any(feature = "icrc-host", feature = "nns-topology-host"))]
impl HostJsonCacheErrorMapper {
    pub const fn new(component: &'static str) -> Self {
        Self { component }
    }
}

#[cfg(any(feature = "icrc-host", feature = "nns-topology-host"))]
impl LoadJsonCacheErrorMapper for HostJsonCacheErrorMapper {
    type Error = HostCacheError;

    fn missing_cache(&self, path: PathBuf) -> Self::Error {
        HostCacheError::missing_cache(self.component, path)
    }

    fn cache_operation(&self, source: CacheFileError) -> Self::Error {
        HostCacheError::operation(self.component, source)
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

///
/// OwnerJsonCacheErrorMapper
///
/// Preserves an owner-specific missing-cache error while routing every other
/// generic JSON cache failure through [`HostCacheError`].
///

#[cfg(feature = "dashboard-host")]
pub struct OwnerJsonCacheErrorMapper<Error> {
    component: &'static str,
    missing_cache: fn(PathBuf) -> Error,
}

#[cfg(feature = "dashboard-host")]
impl<Error> OwnerJsonCacheErrorMapper<Error> {
    /// Build one mapper for a component with specialized missing-cache guidance.
    pub const fn new(component: &'static str, missing_cache: fn(PathBuf) -> Error) -> Self {
        Self {
            component,
            missing_cache,
        }
    }
}

#[cfg(feature = "dashboard-host")]
impl<Error> LoadJsonCacheErrorMapper for OwnerJsonCacheErrorMapper<Error>
where
    Error: From<HostCacheError>,
{
    type Error = Error;

    fn missing_cache(&self, path: PathBuf) -> Self::Error {
        (self.missing_cache)(path)
    }

    fn cache_operation(&self, source: CacheFileError) -> Self::Error {
        HostCacheError::operation(self.component, source).into()
    }

    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error {
        HostCacheError::parse_cache(self.component, path, source).into()
    }

    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error {
        HostCacheError::unsupported_cache_schema_version(self.component, version, expected).into()
    }

    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error {
        HostCacheError::network_mismatch(self.component, requested, actual).into()
    }
}
