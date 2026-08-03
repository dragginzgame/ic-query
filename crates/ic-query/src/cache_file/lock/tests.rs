use super::{
    acquire::{acquire_refresh_lock, inspect_refresh_lock},
    model::RefreshLockRequest,
};
use crate::{cache_file::CacheFileError, test_support::temp_dir};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
};

const NETWORK: &str = "ic";
const STALE_AFTER_SECONDS: u64 = 60;

#[test]
fn new_refresh_lock_records_its_stale_policy() {
    let fixture = LockFixture::new("ic-query-recorded-refresh-lock-policy");

    let guard = acquire_refresh_lock(fixture.request(120)).expect("acquire refresh lock");
    let evidence = inspect_refresh_lock(&fixture.lock_path).expect("inspect refresh lock");

    assert_eq!(evidence.schema_version, 2);
    assert_eq!(evidence.stale_after_seconds, STALE_AFTER_SECONDS);
    assert_eq!(
        evidence.target_path,
        fixture.target_path.display().to_string()
    );
    drop(guard);
    assert!(!fixture.lock_path.exists());
    fixture.cleanup();
}

#[test]
fn corrupted_fresh_refresh_lock_requires_manual_cleanup() {
    let fixture = LockFixture::new("ic-query-corrupted-fresh-refresh-lock");
    fixture.write_lock(r#"{"schema_version":2,"started_at_unix_ms":"60"}"#);

    let err = acquire_refresh_lock(fixture.request(60)).expect_err("corrupted lock is rejected");

    assert_parse_refresh_lock_error(err, &fixture.lock_path);
    assert!(fixture.lock_path.exists());
    fixture.cleanup();
}

#[test]
fn corrupted_stale_refresh_lock_requires_manual_cleanup() {
    let fixture = LockFixture::new("ic-query-corrupted-stale-refresh-lock");
    fixture
        .write_lock(r#"{"schema_version":2,"network":"ic","pid":999999,"started_at_unix_ms":1,"#);

    let err = acquire_refresh_lock(fixture.request(120)).expect_err("corrupted lock is rejected");

    assert_parse_refresh_lock_error(err, &fixture.lock_path);
    assert!(fixture.lock_path.exists());
    fixture.cleanup();
}

#[test]
fn refresh_lock_rejects_unknown_fields() {
    let fixture = LockFixture::new("ic-query-refresh-lock-unknown-field");
    let mut lock = fixture.valid_lock_value(100_000);
    lock["unexpected"] = json!(true);
    fixture.write_lock_value(&lock);

    let err = acquire_refresh_lock(fixture.request(120)).expect_err("unknown field is rejected");

    assert_parse_refresh_lock_error(err, &fixture.lock_path);
    assert!(fixture.lock_path.exists());
    fixture.cleanup();
}

#[test]
fn refresh_lock_rejects_mismatched_identity() {
    let fixture = LockFixture::new("ic-query-refresh-lock-wrong-network");
    let mut lock = fixture.valid_lock_value(100_000);
    lock["network"] = json!("local");
    fixture.write_lock_value(&lock);

    let err = acquire_refresh_lock(fixture.request(120)).expect_err("wrong network is rejected");

    assert!(matches!(err, CacheFileError::InvalidRefreshLock { .. }));
    assert!(fixture.lock_path.exists());
    fixture.cleanup();
}

#[test]
fn stale_valid_refresh_lock_requires_manual_cleanup() {
    let fixture = LockFixture::new("ic-query-stale-valid-refresh-lock");
    fixture.write_valid_lock(1);

    let err = acquire_refresh_lock(fixture.request(120)).expect_err("stale lock is rejected");

    assert!(matches!(err, CacheFileError::StaleRefreshLock { .. }));
    assert!(fixture.lock_path.exists());
    fixture.cleanup();
}

#[test]
fn active_valid_refresh_lock_is_rejected() {
    let fixture = LockFixture::new("ic-query-active-valid-refresh-lock");
    fixture.write_valid_lock(100_000);

    let err = acquire_refresh_lock(fixture.request(120)).expect_err("active lock is rejected");

    match err {
        CacheFileError::RefreshAlreadyInProgress {
            path,
            started_at_unix_ms,
        } => {
            assert_eq!(path, fixture.lock_path);
            assert_eq!(started_at_unix_ms, 100_000);
        }
        other => panic!("unexpected error: {other:?}"),
    }
    assert!(fixture.lock_path.exists());
    fixture.cleanup();
}

#[test]
fn future_dated_refresh_lock_requires_manual_cleanup() {
    let fixture = LockFixture::new("ic-query-future-refresh-lock");
    fixture.write_valid_lock(121_000);

    let err = acquire_refresh_lock(fixture.request(120)).expect_err("future lock is rejected");

    assert!(matches!(err, CacheFileError::InvalidRefreshLock { .. }));
    assert!(fixture.lock_path.exists());
    fixture.cleanup();
}

#[test]
fn existing_refresh_lock_uses_its_recorded_stale_policy() {
    let fixture = LockFixture::new("ic-query-self-describing-refresh-lock");
    fixture.write_valid_lock(100_000);
    let mut request = fixture.request(130);
    request.lock_stale_after_seconds = 10;

    let err = acquire_refresh_lock(request).expect_err("recorded policy keeps lock active");

    assert!(matches!(
        err,
        CacheFileError::RefreshAlreadyInProgress { .. }
    ));
    assert!(fixture.lock_path.exists());
    fixture.cleanup();
}

fn assert_parse_refresh_lock_error(err: CacheFileError, lock_path: &Path) {
    let message = err.to_string();
    match err {
        CacheFileError::ParseRefreshLock { path, .. } => assert_eq!(path, lock_path),
        other => panic!("unexpected error: {other:?}"),
    }
    assert!(message.contains("remove the lock manually"));
}

struct LockFixture {
    root: PathBuf,
    lock_path: PathBuf,
    target_path: PathBuf,
}

impl LockFixture {
    fn new(prefix: &str) -> Self {
        let root = temp_dir(prefix);
        let target_path = root.join("test").join("full.json");
        let lock_path = target_path.with_file_name("full.refresh.lock");
        fs::create_dir_all(lock_path.parent().expect("lock parent")).expect("create lock parent");
        Self {
            root,
            lock_path,
            target_path,
        }
    }

    fn request(&self, now_unix_secs: u64) -> RefreshLockRequest<'_> {
        RefreshLockRequest {
            lock_path: &self.lock_path,
            target_path: &self.target_path,
            network: NETWORK,
            now_unix_secs,
            lock_stale_after_seconds: STALE_AFTER_SECONDS,
        }
    }

    fn write_lock(&self, contents: &str) {
        fs::write(&self.lock_path, contents).expect("write lock");
    }

    fn write_valid_lock(&self, started_at_unix_ms: u64) {
        self.write_lock_value(&self.valid_lock_value(started_at_unix_ms));
    }

    fn valid_lock_value(&self, started_at_unix_ms: u64) -> serde_json::Value {
        json!({
            "schema_version": 2,
            "network": NETWORK,
            "pid": 999_999,
            "started_at_unix_ms": started_at_unix_ms,
            "stale_after_seconds": STALE_AFTER_SECONDS,
            "target_path": self.target_path.display().to_string(),
        })
    }

    fn write_lock_value(&self, value: &serde_json::Value) {
        fs::write(
            &self.lock_path,
            serde_json::to_vec_pretty(value).expect("serialize lock"),
        )
        .expect("write lock");
    }

    fn cleanup(self) {
        let _ = fs::remove_dir_all(self.root);
    }
}
