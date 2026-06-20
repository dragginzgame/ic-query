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
