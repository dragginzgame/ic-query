use super::policy::CacheRefreshReason;
use super::{load_or_refresh_missing_cache, load_or_refresh_stale_cache};
use std::{cell::Cell, path::PathBuf};

#[derive(Debug, Eq, PartialEq)]
enum PolicyError {
    Missing(PathBuf),
    Other,
}

fn missing_path(err: PolicyError) -> Result<PathBuf, PolicyError> {
    match err {
        PolicyError::Missing(path) => Ok(path),
        err @ PolicyError::Other => Err(err),
    }
}

#[test]
fn existing_cache_does_not_refresh() {
    let refreshed = Cell::new(false);

    let loaded = load_or_refresh_missing_cache(
        || Ok::<_, PolicyError>("cached"),
        missing_path,
        |_| {
            refreshed.set(true);
            Ok(())
        },
    );

    assert_eq!(loaded, Ok("cached"));
    assert!(!refreshed.get());
}

#[test]
fn missing_cache_refreshes_then_loads_again() {
    let loads = Cell::new(0);
    let refreshes = Cell::new(0);

    let loaded = load_or_refresh_missing_cache(
        || {
            loads.set(loads.get() + 1);
            if loads.get() == 1 {
                Err(PolicyError::Missing(PathBuf::from("/tmp/missing.json")))
            } else {
                Ok("refreshed")
            }
        },
        missing_path,
        |path| {
            assert_eq!(path, PathBuf::from("/tmp/missing.json"));
            refreshes.set(refreshes.get() + 1);
            Ok(())
        },
    );

    assert_eq!(loaded, Ok("refreshed"));
    assert_eq!(loads.get(), 2);
    assert_eq!(refreshes.get(), 1);
}

#[test]
fn non_missing_error_does_not_refresh() {
    let refreshed = Cell::new(false);

    let loaded = load_or_refresh_missing_cache(
        || Err::<&str, _>(PolicyError::Other),
        missing_path,
        |_| {
            refreshed.set(true);
            Ok(())
        },
    );

    assert_eq!(loaded, Err(PolicyError::Other));
    assert!(!refreshed.get());
}

#[test]
fn stale_cache_refreshes_then_loads_persisted_result() {
    let loads = Cell::new(0);
    let refreshes = Cell::new(0);

    let loaded = load_or_refresh_stale_cache(
        || {
            loads.set(loads.get() + 1);
            Ok::<_, PolicyError>(if loads.get() == 1 { "stale" } else { "fresh" })
        },
        |cached| *cached == "stale",
        missing_path,
        |reason| {
            assert_eq!(reason, CacheRefreshReason::Stale);
            refreshes.set(refreshes.get() + 1);
            Ok(())
        },
    );

    assert_eq!(loaded, Ok("fresh"));
    assert_eq!(loads.get(), 2);
    assert_eq!(refreshes.get(), 1);
}

#[test]
fn stale_policy_reports_missing_path_to_refresh() {
    let loads = Cell::new(0);

    let loaded = load_or_refresh_stale_cache(
        || {
            loads.set(loads.get() + 1);
            if loads.get() == 1 {
                Err(PolicyError::Missing(PathBuf::from("/tmp/missing.json")))
            } else {
                Ok("fresh")
            }
        },
        |_| false,
        missing_path,
        |reason| {
            assert_eq!(
                reason,
                CacheRefreshReason::Missing(PathBuf::from("/tmp/missing.json"))
            );
            Ok(())
        },
    );

    assert_eq!(loaded, Ok("fresh"));
    assert_eq!(loads.get(), 2);
}
