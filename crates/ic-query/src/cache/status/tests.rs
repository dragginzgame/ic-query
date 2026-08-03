use super::*;
use crate::subnet_catalog::parse_utc_timestamp_secs;
use std::{fs, path::Path, time::SystemTime};

#[test]
fn status_reports_managed_unmanaged_and_invalid_caches_without_attempts() {
    let root = temp_dir("ic-query-cache-status");
    write_cache(
        &root.join("nns/ic/subnet-catalog/catalog.json"),
        r#"{"catalog_schema_version":1,"network":"ic","fetched_at":"2026-08-03T00:00:00Z"}"#,
    );
    write_cache(
        &root.join("nns/ic/governance/proposals/full.json"),
        r#"{"schema_version":1,"network":"ic","fetched_at":"2026-08-02T00:00:00Z"}"#,
    );
    write_cache(
        &root.join("sns/ic/root/proposals/full.refresh-attempt.json"),
        r#"{"schema_version":1}"#,
    );
    write_cache(&root.join("nns/ic/node/nodes.json"), "not-json");

    let now = parse_utc_timestamp_secs("2026-08-04T00:00:00Z").expect("timestamp");
    let report =
        build_cache_status_report(&CacheStatusRequest::new(&root, now)).expect("cache status");

    assert_eq!(report.cache_count, 3);
    assert_eq!(report.fresh_count, 1);
    assert_eq!(report.unmanaged_count, 1);
    assert_eq!(report.invalid_count, 1);
    assert!(!report.truncated);
    assert!(
        report
            .caches
            .iter()
            .any(|row| { row.component == "nns/nodes" && row.status == CacheFileStatus::Invalid })
    );
    assert!(
        report
            .caches
            .iter()
            .any(|row| row.component == "nns/governance/proposals")
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn status_reports_active_stale_and_invalid_refresh_locks_without_removing_them() {
    let root = temp_dir("ic-query-refresh-lock-status");
    let now = parse_utc_timestamp_secs("2026-08-04T00:00:00Z").expect("timestamp");
    let active_lock = root.join("nns/ic/node/refresh.lock");
    let stale_lock = root.join("sns/ic/root/proposals/full.refresh.lock");
    let invalid_lock = root.join("nns/ic/subnet-topology/refresh.lock");
    let future_lock = root.join("nns/ic/node-provider/refresh.lock");
    write_refresh_lock(
        &active_lock,
        &root.join("nns/ic/node/nodes.json"),
        now.saturating_sub(30),
        60,
    );
    write_refresh_lock(
        &stale_lock,
        &root.join("sns/ic/root/proposals/full.json"),
        now.saturating_sub(61),
        60,
    );
    write_cache(&invalid_lock, "not-json");
    write_refresh_lock(
        &future_lock,
        &root.join("nns/ic/node-provider/providers.json"),
        now.saturating_add(1),
        60,
    );

    let report =
        build_cache_status_report(&CacheStatusRequest::new(&root, now)).expect("cache status");

    assert_eq!(report.cache_count, 0);
    assert_eq!(report.refresh_lock_count, 4);
    assert_eq!(report.active_refresh_lock_count, 1);
    assert_eq!(report.stale_refresh_lock_count, 1);
    assert_eq!(report.invalid_refresh_lock_count, 2);
    assert!(report.refresh_locks.iter().any(|row| {
        row.status == CacheRefreshLockStatus::Active && row.age_seconds == Some(30)
    }));
    assert!(report.refresh_locks.iter().any(|row| {
        row.status == CacheRefreshLockStatus::Stale
            && row.component == "sns/proposals"
            && row.age_seconds == Some(61)
    }));
    assert!(
        report
            .refresh_locks
            .iter()
            .any(|row| { row.status == CacheRefreshLockStatus::Invalid && row.error.is_some() })
    );
    let text = crate::cache::cache_status_report_text(&report);
    assert!(text.contains("REFRESH LOCKS"));
    assert!(text.contains("STALE AFTER"));
    assert!(active_lock.exists());
    assert!(stale_lock.exists());
    assert!(invalid_lock.exists());
    assert!(future_lock.exists());
    let _ = fs::remove_dir_all(root);
}

fn write_cache(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("parent")).expect("create cache parent");
    fs::write(path, contents).expect("write cache");
}

fn write_refresh_lock(
    lock_path: &Path,
    target_path: &Path,
    started_at_unix_secs: u64,
    stale_after_seconds: u64,
) {
    let contents = serde_json::json!({
        "schema_version": 2,
        "network": "ic",
        "pid": 1234,
        "started_at_unix_ms": started_at_unix_secs.saturating_mul(1_000),
        "stale_after_seconds": stale_after_seconds,
        "target_path": target_path.display().to_string(),
    });
    write_cache(
        lock_path,
        &serde_json::to_string(&contents).expect("serialize refresh lock"),
    );
}

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{nonce}"))
}
