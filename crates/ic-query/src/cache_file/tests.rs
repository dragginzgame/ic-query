use super::load_or_refresh_missing_cache;
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
        "test",
        "https://example.test",
        || Ok::<_, PolicyError>("cached"),
        || {
            refreshed.set(true);
            Ok(())
        },
        missing_path,
    );

    assert_eq!(loaded, Ok("cached"));
    assert!(!refreshed.get());
}

#[test]
fn missing_cache_refreshes_then_loads_again() {
    let loads = Cell::new(0);
    let refreshes = Cell::new(0);

    let loaded = load_or_refresh_missing_cache(
        "test",
        "https://example.test",
        || {
            loads.set(loads.get() + 1);
            if loads.get() == 1 {
                Err(PolicyError::Missing(PathBuf::from("/tmp/missing.json")))
            } else {
                Ok("refreshed")
            }
        },
        || {
            refreshes.set(refreshes.get() + 1);
            Ok(())
        },
        missing_path,
    );

    assert_eq!(loaded, Ok("refreshed"));
    assert_eq!(loads.get(), 2);
    assert_eq!(refreshes.get(), 1);
}

#[test]
fn non_missing_error_does_not_refresh() {
    let refreshed = Cell::new(false);

    let loaded = load_or_refresh_missing_cache(
        "test",
        "https://example.test",
        || Err::<&str, _>(PolicyError::Other),
        || {
            refreshed.set(true);
            Ok(())
        },
        missing_path,
    );

    assert_eq!(loaded, Err(PolicyError::Other));
    assert!(!refreshed.get());
}
