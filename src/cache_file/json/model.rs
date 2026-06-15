use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedJsonReport<T> {
    pub path: PathBuf,
    pub report: T,
}

pub trait JsonCacheReport {
    fn schema_version(&self) -> u32;
    fn network(&self) -> &str;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadJsonCacheRequest<'a> {
    pub path: PathBuf,
    pub network: &'a str,
    pub expected_schema_version: u32,
}
