use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use thiserror::Error as ThisError;

const CACHE_ROOT_ENV: &str = "ICQ_CACHE_ROOT";
const XDG_CACHE_HOME_ENV: &str = "XDG_CACHE_HOME";
const HOME_ENV: &str = "HOME";
const APPLICATION_CACHE_DIR: &str = "ic-query";

///
/// CacheRootError
///
/// Failure to resolve the CLI's user-level cache root.
///

#[derive(Debug, ThisError)]
pub enum CacheRootError {
    #[error(
        "cannot resolve the ic-query cache root: neither {XDG_CACHE_HOME_ENV} nor {HOME_ENV} is set"
    )]
    MissingHome,

    #[error("{variable} must be an absolute path, got {}", path.display())]
    RelativePath {
        variable: &'static str,
        path: PathBuf,
    },
}

pub fn cache_root() -> Result<PathBuf, CacheRootError> {
    resolve_cache_root(
        std::env::var_os(CACHE_ROOT_ENV).as_deref(),
        std::env::var_os(XDG_CACHE_HOME_ENV).as_deref(),
        std::env::var_os(HOME_ENV).as_deref(),
    )
}

fn resolve_cache_root(
    override_root: Option<&OsStr>,
    xdg_cache_home: Option<&OsStr>,
    home: Option<&OsStr>,
) -> Result<PathBuf, CacheRootError> {
    if let Some(path) = nonempty_path(override_root) {
        return absolute_path(CACHE_ROOT_ENV, path);
    }
    if let Some(path) = nonempty_path(xdg_cache_home)
        && path.is_absolute()
    {
        return Ok(path.join(APPLICATION_CACHE_DIR));
    }
    let Some(path) = nonempty_path(home) else {
        return Err(CacheRootError::MissingHome);
    };
    Ok(absolute_path(HOME_ENV, path)?
        .join(".cache")
        .join(APPLICATION_CACHE_DIR))
}

fn nonempty_path(value: Option<&OsStr>) -> Option<&Path> {
    value.filter(|value| !value.is_empty()).map(Path::new)
}

fn absolute_path(variable: &'static str, path: &Path) -> Result<PathBuf, CacheRootError> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Err(CacheRootError::RelativePath {
            variable,
            path: path.to_path_buf(),
        })
    }
}

#[cfg(test)]
mod tests;
