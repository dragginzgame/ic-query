use super::*;
use crate::{
    CacheFileError, cache::CacheRecoveryPolicy, cache_file::write_managed_text_atomically,
    subnet_catalog::parse_utc_timestamp_secs,
};
use std::{fs, path::Path, time::SystemTime};

#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};

#[test]
fn status_separates_header_age_and_recovery_evidence_without_attempts() {
    let root = temp_dir("ic-query-cache-status");
    write_cache(
        &root,
        &root.join("nns/ic/subnet-catalog/catalog.json"),
        r#"{"catalog_schema_version":1,"network":"ic","fetched_at":"2026-08-03T00:00:00Z"}"#,
    );
    write_cache(
        &root,
        &root.join("nns/ic/governance/proposals/full.json"),
        r#"{"schema_version":1,"network":"ic","fetched_at":"2026-08-02T00:00:00Z"}"#,
    );
    write_cache(
        &root,
        &root.join("sns/ic/root/proposals/full.refresh-attempt.json"),
        r#"{"schema_version":1}"#,
    );
    write_cache(
        &root,
        &root.join("sns/ic/root/proposals/full.json"),
        r#"{"schema_version":1,"network":"ic","fetched_at":"not-a-timestamp","domain":"sns","entity":"root","collection":"proposals"}"#,
    );
    write_cache(
        &root,
        &root.join("legacy/ic/full.json"),
        r#"{"schema_version":1,"network":"ic","fetched_at":"2026-08-03T00:00:00Z","domain":"sns","entity":"catalog","collection":"discovery"}"#,
    );
    write_cache(&root, &root.join("nns/ic/node/nodes.json"), "not-json");

    let now = parse_utc_timestamp_secs("2026-08-04T00:00:00Z").expect("timestamp");
    let report =
        build_cache_status_report(&CacheStatusRequest::new(&root, now)).expect("cache status");

    assert_eq!(report.cache_count, 5);
    assert!(!report.family_validation_performed);
    assert_eq!(report.readable_header_count, 4);
    assert_eq!(report.invalid_header_count, 1);
    assert_eq!(report.fresh_count, 1);
    assert_eq!(report.unmanaged_age_count, 2);
    assert_eq!(report.unknown_age_count, 2);
    assert!(!report.truncated);
    let invalid_node = report
        .caches
        .iter()
        .find(|row| row.component == "nns/nodes")
        .expect("invalid node cache row");
    assert_eq!(invalid_node.header_status, CacheHeaderStatus::Invalid);
    assert_eq!(invalid_node.age_status, CacheAgeStatus::Unknown);
    assert_eq!(invalid_node.recovery_policy, CacheRecoveryPolicy::Automatic);
    let nns_proposals = report
        .caches
        .iter()
        .find(|row| row.component == "nns/governance/proposals")
        .expect("NNS proposal cache row");
    assert_eq!(nns_proposals.header_status, CacheHeaderStatus::Readable);
    assert_eq!(nns_proposals.age_status, CacheAgeStatus::Unmanaged);
    assert_eq!(nns_proposals.recovery_policy, CacheRecoveryPolicy::Explicit);
    let sns_proposals = report
        .caches
        .iter()
        .find(|row| row.relative_path == "sns/ic/root/proposals/full.json")
        .expect("SNS proposal cache row");
    assert_eq!(sns_proposals.header_status, CacheHeaderStatus::Readable);
    assert_eq!(sns_proposals.age_status, CacheAgeStatus::Unknown);
    assert_eq!(
        sns_proposals.recovery_policy,
        CacheRecoveryPolicy::MissingOnly
    );
    assert!(sns_proposals.inspection_error.is_some());
    let claimed_sns_catalog = report
        .caches
        .iter()
        .find(|row| row.relative_path == "legacy/ic/full.json")
        .expect("orphaned cache row");
    assert_eq!(
        claimed_sns_catalog.recovery_policy,
        CacheRecoveryPolicy::Unknown
    );
    assert_eq!(claimed_sns_catalog.age_status, CacheAgeStatus::Unmanaged);
    assert_eq!(claimed_sns_catalog.stale_after_seconds, None);
    let text = crate::cache::cache_status_report_text(&report);
    assert!(text.contains("family_validation_performed: no"));
    let header = text
        .lines()
        .find(|line| line.starts_with("HEADER"))
        .expect("cache status table header");
    assert!(header.contains("AGE STATE"));
    assert!(header.contains("RECOVERY"));
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
        &root,
        &active_lock,
        &root.join("nns/ic/node/nodes.json"),
        now.saturating_sub(30),
        60,
    );
    write_refresh_lock(
        &root,
        &stale_lock,
        &root.join("sns/ic/root/proposals/full.json"),
        now.saturating_sub(61),
        60,
    );
    write_cache(&root, &invalid_lock, "not-json");
    write_refresh_lock(
        &root,
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

#[cfg(unix)]
#[test]
fn status_inventory_rejects_symlinks_inside_the_managed_root() {
    let root = temp_dir("ic-query-cache-status-symlink");
    write_cache(&root, &root.join("seed.json"), "{}");
    symlink(&root, root.join("linked-cache")).expect("create managed-root symlink");

    let error = build_cache_status_report(&CacheStatusRequest::new(&root, 1_700_000_000))
        .expect_err("symlinked inventory entry rejected");

    assert!(matches!(
        error,
        CacheStatusError::CacheOperation(CacheFileError::Confinement { .. })
    ));
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn status_inventory_rejects_unsafe_managed_file_modes() {
    let root = temp_dir("ic-query-cache-status-mode");
    let path = root.join("nns/ic/node/nodes.json");
    write_cache(&root, &path, "{}");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("widen cache mode");

    let error = build_cache_status_report(&CacheStatusRequest::new(&root, 1_700_000_000))
        .expect_err("unsafe inventory file rejected");

    assert!(matches!(
        error,
        CacheStatusError::CacheOperation(CacheFileError::UnsafeManagedPermissions {
            actual_mode: 0o644,
            ..
        })
    ));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("restore cache mode");
    let _ = fs::remove_dir_all(root);
}

fn write_cache(root: &Path, path: &Path, contents: &str) {
    write_managed_text_atomically(root, path, contents).expect("write cache");
}

fn write_refresh_lock(
    root: &Path,
    lock_path: &Path,
    target_path: &Path,
    started_at_unix_secs: u64,
    stale_after_seconds: u64,
) {
    let contents = serde_json::json!({
        "schema_version": 1,
        "network": "ic",
        "pid": 1234,
        "started_at_unix_ms": started_at_unix_secs.saturating_mul(1_000),
        "stale_after_seconds": stale_after_seconds,
        "target_path": target_path.display().to_string(),
    });
    write_cache(
        root,
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
