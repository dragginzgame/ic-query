use super::CacheFileError;
use serde::de::DeserializeOwned;
use std::{fs, io, path::PathBuf};

///
/// CachedJsonReport
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedJsonReport<T> {
    pub path: PathBuf,
    pub report: T,
}

///
/// JsonCacheReport
///
pub trait JsonCacheReport {
    fn schema_version(&self) -> u32;
    fn network(&self) -> &str;
}

///
/// LoadJsonCacheRequest
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadJsonCacheRequest<'a> {
    pub path: PathBuf,
    pub network: &'a str,
    pub expected_schema_version: u32,
}

///
/// LoadJsonCacheErrorMapper
///
pub trait LoadJsonCacheErrorMapper {
    type Error;

    fn missing_cache(&self, path: PathBuf) -> Self::Error;
    fn read_cache(&self, path: PathBuf, source: io::Error) -> Self::Error;
    fn parse_cache(&self, path: PathBuf, source: serde_json::Error) -> Self::Error;
    fn unsupported_schema(&self, version: u32, expected: u32) -> Self::Error;
    fn network_mismatch(&self, requested: String, actual: String) -> Self::Error;
}

pub fn announce_cache_refresh(component: &str, path: &std::path::Path, source_endpoint: &str) {
    eprintln!(
        "{component} cache missing at {}; calling {source_endpoint} to refresh/create cache",
        path.display()
    );
}

pub fn load_json_cache<T, Errors>(
    request: LoadJsonCacheRequest<'_>,
    errors: Errors,
) -> Result<CachedJsonReport<T>, Errors::Error>
where
    T: DeserializeOwned + JsonCacheReport,
    Errors: LoadJsonCacheErrorMapper,
{
    let path = request.path;
    if !path.is_file() {
        return Err(errors.missing_cache(path));
    }
    let data =
        fs::read_to_string(&path).map_err(|source| errors.read_cache(path.clone(), source))?;
    let report = serde_json::from_str::<T>(&data)
        .map_err(|source| errors.parse_cache(path.clone(), source))?;
    let actual_schema_version = report.schema_version();
    if actual_schema_version != request.expected_schema_version {
        return Err(
            errors.unsupported_schema(actual_schema_version, request.expected_schema_version)
        );
    }
    let actual_network = report.network();
    if actual_network != request.network {
        return Err(
            errors.network_mismatch(request.network.to_string(), actual_network.to_string())
        );
    }
    Ok(CachedJsonReport { path, report })
}

pub(super) fn read_json_file<T>(path: &std::path::Path) -> Result<T, CacheFileError>
where
    T: DeserializeOwned,
{
    let data = fs::read(path).map_err(|source| CacheFileError::ReadRefreshLock {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_slice(&data).map_err(|source| CacheFileError::ParseRefreshLock {
        path: path.to_path_buf(),
        source,
    })
}
