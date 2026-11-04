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
