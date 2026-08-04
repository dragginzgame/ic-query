use super::policy::CacheRefreshReason;
use super::{
    HostCacheError, host_cache_refresh_reason, load_or_refresh_cache_with_error_policy,
    load_or_refresh_missing_cache, load_or_refresh_stale_cache_with_error_policy,
};
use std::{cell::Cell, path::PathBuf};

#[derive(Debug, Eq, PartialEq)]
enum PolicyError {
    Missing(PathBuf),
    Invalid(PathBuf),
    Other,
}

fn missing_path(err: PolicyError) -> Result<PathBuf, PolicyError> {
    match err {
        PolicyError::Missing(path) => Ok(path),
        err @ (PolicyError::Invalid(_) | PolicyError::Other) => Err(err),
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

    let loaded = load_or_refresh_stale_cache_with_error_policy(
        || {
            loads.set(loads.get() + 1);
            Ok::<_, PolicyError>(if loads.get() == 1 { "stale" } else { "fresh" })
        },
        |cached| *cached == "stale",
        |error| missing_path(error).map(CacheRefreshReason::Missing),
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

    let loaded = load_or_refresh_stale_cache_with_error_policy(
        || {
            loads.set(loads.get() + 1);
            if loads.get() == 1 {
                Err(PolicyError::Missing(PathBuf::from("/tmp/missing.json")))
            } else {
                Ok("fresh")
            }
        },
        |_| false,
        |error| missing_path(error).map(CacheRefreshReason::Missing),
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

#[test]
fn owner_error_policy_refreshes_invalid_cache_then_loads_again() {
    let loads = Cell::new(0);
    let refreshes = Cell::new(0);
    let path = PathBuf::from("/tmp/invalid.json");

    let loaded = load_or_refresh_stale_cache_with_error_policy(
        || {
            loads.set(loads.get() + 1);
            if loads.get() == 1 {
                Err(PolicyError::Invalid(path.clone()))
            } else {
                Ok("refreshed")
            }
        },
        |_| false,
        |error| match error {
            PolicyError::Invalid(path) => Ok(CacheRefreshReason::Invalid(path)),
            error => Err(error),
        },
        |reason| {
            assert_eq!(reason, CacheRefreshReason::Invalid(path.clone()));
            refreshes.set(refreshes.get() + 1);
            Ok(())
        },
    );

    assert_eq!(loaded, Ok("refreshed"));
    assert_eq!(loads.get(), 2);
    assert_eq!(refreshes.get(), 1);
}

#[test]
fn non_stale_owner_policy_refreshes_invalid_cache_then_loads_again() {
    let loads = Cell::new(0);
    let path = PathBuf::from("/tmp/invalid.json");

    let loaded = load_or_refresh_cache_with_error_policy(
        || {
            loads.set(loads.get() + 1);
            if loads.get() == 1 {
                Err(PolicyError::Invalid(path.clone()))
            } else {
                Ok("refreshed")
            }
        },
        |error| match error {
            PolicyError::Invalid(path) => Ok(CacheRefreshReason::Invalid(path)),
            error => Err(error),
        },
        |reason| {
            assert_eq!(reason, CacheRefreshReason::Invalid(path.clone()));
            Ok(())
        },
    );

    assert_eq!(loaded, Ok("refreshed"));
    assert_eq!(loads.get(), 2);
}

#[test]
fn shared_host_classifier_recovers_content_errors_but_preserves_read_errors() {
    let expected = PathBuf::from("/tmp/cache.json");
    let missing = PathBuf::from("/tmp/missing.json");
    let reason = host_cache_refresh_reason(
        HostCacheError::missing_cache("fixture", missing.clone()),
        &expected,
    )
    .expect("missing cache is recoverable");
    assert_eq!(reason, CacheRefreshReason::Missing(missing));

    let parse_error =
        serde_json::from_str::<serde_json::Value>("not-json").expect_err("invalid JSON fixture");
    let reason = host_cache_refresh_reason(
        HostCacheError::parse_cache("fixture", expected.clone(), parse_error),
        &expected,
    )
    .expect("parse error is recoverable");
    assert_eq!(reason, CacheRefreshReason::Invalid(expected.clone()));

    let reason = host_cache_refresh_reason(
        HostCacheError::invalid_cache("fixture", expected.clone(), "count mismatch".to_string()),
        &expected,
    )
    .expect("semantic error is recoverable");
    assert_eq!(reason, CacheRefreshReason::Invalid(expected.clone()));

    for error in [
        HostCacheError::unsupported_cache_schema_version("fixture", 2, 1),
        HostCacheError::network_mismatch("fixture", "ic".to_string(), "local".to_string()),
    ] {
        let reason = host_cache_refresh_reason(error, &expected)
            .expect("header content error is recoverable");
        assert_eq!(reason, CacheRefreshReason::Invalid(expected.clone()));
    }

    let read_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "fixture");
    let error = host_cache_refresh_reason(
        HostCacheError::read_cache("fixture", expected.clone(), read_error),
        &expected,
    )
    .expect_err("read error remains visible");
    assert!(matches!(error, HostCacheError::ReadCache { path, .. } if path == expected));
}
