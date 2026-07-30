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
mod tests {
    use super::{CacheRootError, resolve_cache_root};
    use std::{
        ffi::OsStr,
        path::{Path, PathBuf},
    };

    #[test]
    fn explicit_cache_root_wins() {
        assert_eq!(
            resolve_cache_root(
                Some(OsStr::new("/var/cache/icq")),
                Some(OsStr::new("/xdg")),
                Some(OsStr::new("/home/test")),
            )
            .expect("cache root"),
            PathBuf::from("/var/cache/icq")
        );
    }

    #[test]
    fn xdg_cache_home_is_preferred_to_home() {
        assert_eq!(
            resolve_cache_root(
                None,
                Some(OsStr::new("/xdg/cache")),
                Some(OsStr::new("/home/test")),
            )
            .expect("cache root"),
            PathBuf::from("/xdg/cache/ic-query")
        );
    }

    #[test]
    fn home_uses_standard_cache_directory() {
        assert_eq!(
            resolve_cache_root(None, None, Some(OsStr::new("/home/test"))).expect("cache root"),
            PathBuf::from("/home/test/.cache/ic-query")
        );
    }

    #[test]
    fn relative_xdg_cache_home_is_ignored() {
        assert_eq!(
            resolve_cache_root(
                None,
                Some(OsStr::new(".cache")),
                Some(OsStr::new("/home/test")),
            )
            .expect("cache root"),
            PathBuf::from("/home/test/.cache/ic-query")
        );
    }

    #[test]
    fn empty_values_are_unset() {
        assert_eq!(
            resolve_cache_root(
                Some(OsStr::new("")),
                Some(OsStr::new("")),
                Some(OsStr::new("/home/test")),
            )
            .expect("cache root"),
            PathBuf::from("/home/test/.cache/ic-query")
        );
    }

    #[test]
    fn relative_paths_are_rejected() {
        let error = resolve_cache_root(
            Some(OsStr::new(".cache/ic-query")),
            None,
            Some(OsStr::new("/home/test")),
        )
        .expect_err("relative override should fail");

        assert!(matches!(
            error,
            CacheRootError::RelativePath {
                variable: "ICQ_CACHE_ROOT",
                path,
            } if path == Path::new(".cache/ic-query")
        ));
    }

    #[test]
    fn missing_home_is_reported() {
        assert!(matches!(
            resolve_cache_root(None, None, None),
            Err(CacheRootError::MissingHome)
        ));
    }
}
