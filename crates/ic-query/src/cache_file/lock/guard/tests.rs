use super::RefreshLockGuard;
use crate::tests::temp_dir;
use std::fs::{self, File};

#[test]
fn drop_releases_lock_even_with_a_duplicate_descriptor() {
    let root = temp_dir("ic-query-drop-duplicated-refresh-lock");
    let path = root.join("refresh.lock");
    let file = File::create(&path).unwrap();
    ic_host_fs::durable::try_lock_exclusive(&file).unwrap();
    let duplicate = file.try_clone().unwrap();
    let guard = RefreshLockGuard::new(file, path.clone());
    let observer = File::open(&path).unwrap();
    assert_eq!(
        ic_host_fs::durable::try_lock_exclusive(&observer)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::WouldBlock
    );

    drop(guard);

    ic_host_fs::durable::try_lock_exclusive(&observer).unwrap();
    ic_host_fs::durable::unlock(&observer).unwrap();
    drop(duplicate);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn finish_releases_duplicates_and_preserves_action_results() {
    let root = temp_dir("ic-query-finish-duplicated-refresh-lock");
    let path = root.join("refresh.lock");
    for outcome in [Ok(7), Err("collection failed")] {
        let file = File::create(&path).unwrap();
        ic_host_fs::durable::try_lock_exclusive(&file).unwrap();
        let duplicate = file.try_clone().unwrap();
        let guard = RefreshLockGuard::new(file, path.clone());

        assert_eq!(guard.finish(outcome, |_| "lock release failed"), outcome);

        let observer = File::open(&path).unwrap();
        ic_host_fs::durable::try_lock_exclusive(&observer).unwrap();
        ic_host_fs::durable::unlock(&observer).unwrap();
        drop(duplicate);
    }
    fs::remove_dir_all(root).unwrap();
}
