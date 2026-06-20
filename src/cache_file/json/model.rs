use std::path::PathBuf;

///
/// CachedJsonReport
///
/// Loaded JSON cache report paired with the file path it came from.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedJsonReport<T> {
    pub path: PathBuf,
    pub report: T,
}

///
/// JsonCacheReport
///
/// Minimal metadata every JSON cache report exposes for validation.
///

pub trait JsonCacheReport {
    fn schema_version(&self) -> u32;
    fn network(&self) -> &str;
}

///
/// LoadJsonCacheRequest
///
/// Inputs needed to load and validate one JSON cache report.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadJsonCacheRequest<'a> {
    pub path: PathBuf,
    pub network: &'a str,
    pub expected_schema_version: u32,
}
