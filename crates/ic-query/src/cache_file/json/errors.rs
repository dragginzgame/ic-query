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
    /// Error returned by the cache owner.
    type Error;

    /// Map a missing managed cache.
    fn missing_cache(&self, path: PathBuf) -> Self::Error;
    /// Map a capability-rooted cache operation failure.
    fn cache_operation(&self, source: CacheFileError) -> Self::Error;
    /// Map content exceeding the owner-selected byte ceiling.
    fn cache_too_large(&self, path: PathBuf, actual: u64, maximum: u64) -> Self::Error;
    /// Map invalid JSON or UTF-8 content.
    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error;
    /// Map an unsupported schema identifier.
    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error;
    /// Map a mismatched network identity.
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
    /// Select the component retained in every cache error.
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

    fn cache_too_large(&self, path: PathBuf, actual: u64, maximum: u64) -> Self::Error {
        HostCacheError::CacheTooLarge {
            component: self.component,
            path,
            actual,
            maximum,
        }
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

#[cfg(any(feature = "dashboard-host", feature = "nns-host", feature = "sns-host"))]
pub struct OwnerJsonCacheErrorMapper<Error> {
    component: &'static str,
    missing_cache: fn(PathBuf) -> Error,
}

#[cfg(any(feature = "dashboard-host", feature = "nns-host", feature = "sns-host"))]
impl<Error> OwnerJsonCacheErrorMapper<Error> {
    /// Build one mapper for a component with specialized missing-cache guidance.
    pub const fn new(component: &'static str, missing_cache: fn(PathBuf) -> Error) -> Self {
        Self {
            component,
            missing_cache,
        }
    }
}

#[cfg(any(feature = "dashboard-host", feature = "nns-host", feature = "sns-host"))]
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

    fn cache_too_large(&self, path: PathBuf, actual: u64, maximum: u64) -> Self::Error {
        HostCacheError::CacheTooLarge {
            component: self.component,
            path,
            actual,
            maximum,
        }
        .into()
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
