use super::*;
use crate::test_support::temp_dir;
use crate::{
    CacheFileError, cache::CacheRecoveryPolicy, cache_file::write_managed_text_atomically,
    subnet_catalog::parse_utc_timestamp_secs,
};
use std::{fs, path::Path};

#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};

#[test]
fn status_lists_history_transcripts_and_locks_without_reading_their_payload() {
    let root = temp_dir("ic-query-cache-status-registry-history");
    let history_path = root.join("nns/ic/subnet-catalog/history.json");
    write_cache(
        &root,
        &history_path,
        r#"{"schema_version":1,"network":"ic","fetched_at":"2026-10-01T00:00:00Z","checkpoints":[unread payload"#,
    );
    let lock_path = root.join("nns/ic/subnet-catalog/history.lock");
    write_cache(&root, &lock_path, "");
    let now = parse_utc_timestamp_secs("2026-10-02T00:00:00Z").unwrap();
    let report =
        build_cache_status_report(&CacheStatusRequest::new(&root, now)).expect("history inventory");
    assert_eq!(report.cache_count, 1);
    let row = &report.caches[0];
    assert_eq!(row.component, "nns/registry-history");
    assert_eq!(row.header_status, CacheHeaderStatus::Readable);
    assert_eq!(row.age_status, CacheAgeStatus::Unmanaged);
    assert_eq!(row.recovery_policy, CacheRecoveryPolicy::Explicit);
    assert_eq!(row.schema_version, Some(1));
    assert_eq!(report.refresh_lock_count, 1);
    assert_eq!(report.refresh_locks[0].component, "nns/registry-history");
    assert!(!report.family_validation_performed);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn status_preserves_header_failures_before_an_unread_payload() {
    let root = temp_dir("ic-query-cache-status-invalid-prefix");
    let history = root.join("nns/ic/subnet-catalog/history.json");
    for contents in [
        r#"{"schema_version":"invalid","checkpoints":["#,
        r#"{"schema_version":1,"schema_version":1,"checkpoints":["#,
        r#"{"schema_version":1,"network":123,"checkpoints":["#,
        r#"{"network":"ic","checkpoints":["#,
    ] {
        write_cache(&root, &history, contents);
        let report = build_cache_status_report(&CacheStatusRequest::new(&root, 0)).unwrap();
        assert_eq!(report.cache_count, 1);
        assert_eq!(report.caches[0].header_status, CacheHeaderStatus::Invalid);
        assert!(report.caches[0].inspection_error.is_some());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn status_bounds_registered_files_while_allowing_large_unread_history_payloads() {
    let root = temp_dir("ic-query-cache-status-byte-bounds");
    let catalog_path = root.join("nns/ic/subnet-catalog/catalog.json");
    let history_path = root.join("nns/ic/subnet-catalog/history.json");
    let size_bytes = 64 * 1024 * 1024 + 1;
    for (path, contents) in [
        (
            &catalog_path,
            r#"{"catalog_schema_version":1,"network":"ic","fetched_at":"2026-10-01T00:00:00Z"}"#,
        ),
        (
            &history_path,
            r#"{"schema_version":1,"network":"ic","fetched_at":"2026-10-01T00:00:00Z","checkpoints":["#,
        ),
    ] {
        write_cache(&root, path, contents);
        fs::OpenOptions::new()
            .write(true)
            .open(path)
            .unwrap()
            .set_len(size_bytes)
            .unwrap();
    }
    let now = parse_utc_timestamp_secs("2026-10-02T00:00:00Z").unwrap();
    let report = build_cache_status_report(&CacheStatusRequest::new(&root, now)).unwrap();
    assert_eq!(report.cache_count, 2);
    for row in &report.caches {
        assert_eq!(row.size_bytes, size_bytes);
        if row.component == "nns/registry-history" {
            assert_eq!(row.header_status, CacheHeaderStatus::Readable);
            assert_eq!(row.age_status, CacheAgeStatus::Unmanaged);
        } else {
            assert_eq!(row.header_status, CacheHeaderStatus::Invalid);
            assert!(
                row.inspection_error
                    .as_deref()
                    .unwrap()
                    .contains("67108864")
            );
        }
    }
    fs::remove_dir_all(root).unwrap();
}

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
        &root.join("unmanaged/ic/full.json"),
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
        .find(|row| row.relative_path == "unmanaged/ic/full.json")
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
fn status_reports_active_available_and_invalid_coordination_files() {
    let root = temp_dir("ic-query-refresh-lock-status");
    let active_lock = root.join("nns/ic/node/refresh.lock");
    let available_lock = root.join("sns/ic/root/proposals/full.refresh.lock");
    let invalid_lock = root.join("nns/ic/subnet-topology/refresh.lock");
    for path in [&active_lock, &available_lock] {
        write_cache(&root, path, "");
    }
    write_cache(&root, &invalid_lock, "invalid coordination content");
    let file = ic_host_fs::durable::open_existing_regular_lock_file(&active_lock).unwrap();
    ic_host_fs::durable::try_lock_exclusive(&file).unwrap();
    let report = build_cache_status_report(&CacheStatusRequest::new(&root, 0)).unwrap();
    assert_eq!(report.refresh_lock_count, 3);
    assert_eq!(report.active_refresh_lock_count, 1);
    assert_eq!(report.available_refresh_lock_count, 1);
    assert_eq!(report.invalid_refresh_lock_count, 1);
    let text = crate::cache::cache_status_report_text(&report);
    assert!(text.contains("REFRESH LOCKS"));
    assert!(text.contains("available"));
    assert!(text.contains("active"));
    assert!(active_lock.exists());
    ic_host_fs::durable::unlock(&file).unwrap();
    drop(file);
    let unlocked = build_cache_status_report(&CacheStatusRequest::new(&root, 0)).unwrap();
    assert_eq!(unlocked.active_refresh_lock_count, 0);
    assert_eq!(unlocked.available_refresh_lock_count, 2);
    fs::remove_dir_all(root).unwrap();
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
    fs::set_permissions(&path, fs::Permissions::from_mode(0o664)).expect("widen cache mode");

    let error = build_cache_status_report(&CacheStatusRequest::new(&root, 1_700_000_000))
        .expect_err("unsafe inventory file rejected");

    assert!(matches!(
        error,
        CacheStatusError::CacheOperation(CacheFileError::UnsafeManagedPermissions {
            actual_mode: 0o664,
            ..
        })
    ));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("restore cache mode");
    let _ = fs::remove_dir_all(root);
}

fn write_cache(root: &Path, path: &Path, contents: &str) {
    write_managed_text_atomically(root, path, contents).expect("write cache");
}

#[test]
fn status_header_reads_share_a_budget_even_when_the_first_header_is_invalid() {
    let root = temp_dir("ic-query-status-aggregate-budget");
    let first = root.join("sns/ic/catalog/discovery/full.json");
    let second = root.join("ic/ic/nodes/operational-status/full.json");
    let valid = r#"{"schema_version":1,"network":"ic","fetched_at":"2026-10-01T00:00:00Z"}"#;
    write_cache(&root, &second, valid);
    for contents in [valid, "{invalid"] {
        write_cache(&root, &first, contents);
        let mut budget = crate::cache_file::ManagedReadBudget::new(contents.len() as u64);
        let row = header::cache_status_row(&root, &first, 0, &mut budget).unwrap();
        assert_eq!(
            row.header_status,
            if contents == valid {
                CacheHeaderStatus::Readable
            } else {
                CacheHeaderStatus::Invalid
            }
        );
        let error = header::cache_status_row(&root, &second, 0, &mut budget).unwrap_err();
        assert!(matches!(
            error,
            CacheFileError::ScanLimitExceeded {
                resource: "inspection bytes",
                ..
            }
        ));
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn status_stream_budget_charges_only_the_inspected_history_prefix() {
    let root = temp_dir("ic-query-status-prefix-budget");
    let path = root.join("nns/ic/subnet-catalog/history.json");
    write_cache(&root, &path, r#"{"schema_version":1,"checkpoints":["#);
    fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_len(128 * 1024 * 1024)
        .unwrap();
    let mut budget = crate::cache_file::ManagedReadBudget::new(64 * 1024);
    let row = header::cache_status_row(&root, &path, 0, &mut budget).unwrap();
    assert_eq!(row.header_status, CacheHeaderStatus::Readable);
    fs::remove_dir_all(root).unwrap();
}
