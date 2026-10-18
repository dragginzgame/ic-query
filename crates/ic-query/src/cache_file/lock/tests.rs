use super::{
    acquire::{acquire_refresh_lock, inspect_refresh_lock},
    model::RefreshLockRequest,
};
use crate::{
    cache_file::{CacheFileError, create_managed_parent_directory, write_managed_text_atomically},
    test_support::temp_dir,
};
#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

struct LockFixture {
    root: PathBuf,
    path: PathBuf,
}

impl LockFixture {
    fn new(name: &str) -> Self {
        let root = temp_dir(name);
        let path = root.join("nns/ic/refresh.lock");
        Self { root, path }
    }
    fn request(&self) -> RefreshLockRequest<'_> {
        RefreshLockRequest {
            cache_root: &self.root,
            lock_path: &self.path,
        }
    }
}

#[test]
fn descriptor_exclusion_preserves_persistent_identity_and_releases_on_close() {
    let fixture = LockFixture::new("ic-query-descriptor-lock");
    let guard = acquire_refresh_lock(fixture.request()).unwrap();
    assert!(inspect_refresh_lock(&fixture.root, &fixture.path).unwrap());
    assert!(matches!(
        acquire_refresh_lock(fixture.request()),
        Err(CacheFileError::RefreshAlreadyInProgress { .. })
    ));
    assert_eq!(fs::read(&fixture.path).unwrap(), Vec::<u8>::new());
    #[cfg(unix)]
    assert_eq!(
        fs::metadata(&fixture.path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    drop(guard);
    assert!(!inspect_refresh_lock(&fixture.root, &fixture.path).unwrap());
    let next = acquire_refresh_lock(fixture.request()).unwrap();
    assert!(fixture.path.exists());
    drop(next);
    fs::remove_dir_all(fixture.root).unwrap();
}

#[cfg(feature = "subnet-catalog-host")]
#[test]
fn guarded_refresh_releases_after_success_and_preserves_action_failure() {
    let fixture = LockFixture::new("ic-query-guarded-refresh-outcome");
    for outcome in [Ok(7), Err("collection failed")] {
        let result = super::run::with_refresh_lock(
            fixture.request(),
            |_| "cache failure",
            || {
                assert!(inspect_refresh_lock(&fixture.root, &fixture.path).unwrap());
                outcome
            },
        );
        assert_eq!(result, outcome);
        assert!(!inspect_refresh_lock(&fixture.root, &fixture.path).unwrap());
    }
    fs::remove_dir_all(fixture.root).unwrap();
}

#[cfg(feature = "subnet-catalog-host")]
#[test]
fn async_refresh_releases_after_success_and_preserves_action_failure() {
    let fixture = LockFixture::new("ic-query-async-refresh-outcome");
    for outcome in [Ok(7), Err("collection failed")] {
        let result = futures::executor::block_on(super::run::with_refresh_lock_async(
            fixture.request(),
            |_| "cache failure",
            async || {
                assert!(inspect_refresh_lock(&fixture.root, &fixture.path).unwrap());
                outcome
            },
        ));
        assert_eq!(result, outcome);
        assert!(!inspect_refresh_lock(&fixture.root, &fixture.path).unwrap());
    }
    fs::remove_dir_all(fixture.root).unwrap();
}

#[cfg(feature = "subnet-catalog-host")]
#[test]
fn dropping_a_pending_refresh_releases_its_lock() {
    let fixture = LockFixture::new("ic-query-cancelled-refresh");
    let mut refresh = Box::pin(super::run::with_refresh_lock_async(
        fixture.request(),
        |_| "cache failure",
        async || std::future::pending::<Result<(), &str>>().await,
    ));
    let mut context = std::task::Context::from_waker(futures::task::noop_waker_ref());
    assert!(std::future::Future::poll(refresh.as_mut(), &mut context).is_pending());
    assert!(inspect_refresh_lock(&fixture.root, &fixture.path).unwrap());
    drop(refresh);
    assert!(!inspect_refresh_lock(&fixture.root, &fixture.path).unwrap());
    fs::remove_dir_all(fixture.root).unwrap();
}

#[test]
fn invalid_coordination_content_is_preserved_without_acquisition() {
    let fixture = LockFixture::new("ic-query-invalid-lock");
    for contents in ["occupied", "{}", "\0"] {
        write_managed_text_atomically(&fixture.root, &fixture.path, contents).unwrap();
        assert!(matches!(
            acquire_refresh_lock(fixture.request()),
            Err(CacheFileError::InvalidRefreshLock { .. })
        ));
        assert_eq!(fs::read(&fixture.path).unwrap(), contents.as_bytes());
    }
    fs::remove_dir_all(fixture.root).unwrap();
}

#[cfg(unix)]
#[test]
fn lock_admission_preserves_symlink_targets_and_unsafe_permissions() {
    let fixture = LockFixture::new("ic-query-lock-admission");
    create_managed_parent_directory(&fixture.root, &fixture.path).unwrap();
    let outside = fixture.root.join("outside");
    fs::write(&outside, "preserve").unwrap();
    symlink(&outside, &fixture.path).unwrap();
    assert!(acquire_refresh_lock(fixture.request()).is_err());
    assert_eq!(fs::read_to_string(&outside).unwrap(), "preserve");
    fs::remove_file(&fixture.path).unwrap();
    fs::write(&fixture.path, "").unwrap();
    fs::set_permissions(&fixture.path, fs::Permissions::from_mode(0o666)).unwrap();
    assert!(matches!(
        acquire_refresh_lock(fixture.request()),
        Err(CacheFileError::UnsafeManagedPermissions { .. })
    ));
    fs::remove_dir_all(fixture.root).unwrap();
}

#[test]
fn descriptor_lock_child() {
    let Some(root) = std::env::var_os("IC_QUERY_LOCK_CHILD_ROOT") else {
        return;
    };
    let root = Path::new(&root);
    let _guard = acquire_refresh_lock(RefreshLockRequest {
        cache_root: root,
        lock_path: &root.join("nns/ic/refresh.lock"),
    })
    .unwrap();
    fs::write(root.join("ready"), "ready").unwrap();
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(15) {
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn process_death_releases_kernel_ownership_without_removing_the_sidecar() {
    let fixture = LockFixture::new("ic-query-lock-process-death");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "cache_file::lock::tests::descriptor_lock_child",
            "--nocapture",
        ])
        .env("IC_QUERY_LOCK_CHILD_ROOT", &fixture.root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let started = Instant::now();
    while !fixture.root.join("ready").exists() && started.elapsed() < Duration::from_secs(5) {
        assert!(child.try_wait().unwrap().is_none());
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(fixture.root.join("ready").exists());
    assert!(matches!(
        acquire_refresh_lock(fixture.request()),
        Err(CacheFileError::RefreshAlreadyInProgress { .. })
    ));
    child.kill().unwrap();
    child.wait().unwrap();
    let guard = acquire_refresh_lock(fixture.request()).expect("dead process releases ownership");
    assert!(fixture.path.exists());
    drop(guard);
    fs::remove_dir_all(fixture.root).unwrap();
}
