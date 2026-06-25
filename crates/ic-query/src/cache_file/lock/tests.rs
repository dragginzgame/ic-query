use super::{acquire::acquire_refresh_lock, model::RefreshLockRequest};
use crate::{cache_file::CacheFileError, test_support::temp_dir};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
};

const NETWORK: &str = "ic";
const STALE_AFTER_SECONDS: u64 = 60;

#[test]
fn corrupted_fresh_refresh_lock_requires_manual_cleanup() {
    let fixture = LockFixture::new("ic-query-corrupted-fresh-refresh-lock");
    fixture.write_lock(r#"{"schema_version":1,"started_at_unix_ms":"60"}"#);

    let err = acquire_refresh_lock(fixture.request(60)).expect_err("corrupted lock is rejected");

    assert_parse_refresh_lock_error(err, &fixture.lock_path);
    assert!(fixture.lock_path.exists());
    fixture.cleanup();
}

#[test]
fn corrupted_stale_refresh_lock_requires_manual_cleanup() {
    let fixture = LockFixture::new("ic-query-corrupted-stale-refresh-lock");
    fixture
        .write_lock(r#"{"schema_version":1,"network":"ic","pid":999999,"started_at_unix_ms":1,"#);

    let err = acquire_refresh_lock(fixture.request(120)).expect_err("corrupted lock is rejected");

    assert_parse_refresh_lock_error(err, &fixture.lock_path);
    assert!(fixture.lock_path.exists());
    fixture.cleanup();
}

#[test]
fn stale_valid_refresh_lock_is_replaced() {
    let fixture = LockFixture::new("ic-query-stale-valid-refresh-lock");
    fixture.write_valid_lock(1);

    let guard = acquire_refresh_lock(fixture.request(120)).expect("stale lock is replaced");
    let lock: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.lock_path).expect("read replaced lock"))
            .expect("parse replaced lock");

    assert_eq!(lock["started_at_unix_ms"], 120_000);
    drop(guard);
    assert!(!fixture.lock_path.exists());
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
        let target_path = root.join(".icq").join("test").join("full.json");
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
        fs::write(
            &self.lock_path,
            serde_json::to_vec_pretty(&json!({
                "schema_version": 1,
                "network": NETWORK,
                "pid": 999_999,
                "started_at_unix_ms": started_at_unix_ms,
                "target_path": self.target_path.display().to_string(),
            }))
            .expect("serialize lock"),
        )
        .expect("write lock");
    }

    fn cleanup(self) {
        let _ = fs::remove_dir_all(self.root);
    }
}
