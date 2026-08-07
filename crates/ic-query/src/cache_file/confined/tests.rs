use super::*;
use crate::test_support::temp_dir;
use std::{
    fs,
    io::{self, Write},
    path::Path,
};

#[cfg(unix)]
use std::os::unix::fs::{PermissionsExt, symlink};

#[test]
fn managed_round_trip_uses_owner_only_modes() {
    let root = temp_dir("ic-query-confined-round-trip");
    let path = root.join("nns/ic/report.json");

    write_managed_text_atomically(&root, &path, "evidence").expect("write managed cache");

    assert_eq!(
        read_managed_text(&root, &path).expect("read managed cache"),
        Some("evidence".to_string())
    );
    #[cfg(unix)]
    {
        assert_eq!(
            fs::metadata(&root)
                .expect("root metadata")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&path)
                .expect("file metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    let _ = fs::remove_dir_all(root);
}

#[test]
fn bounded_managed_read_accepts_exact_limit_and_rejects_larger_file() {
    let root = temp_dir("ic-query-confined-bounded-read");
    let path = root.join("nns/ic/archive/object.json");
    write_managed_text_atomically(&root, &path, "12345678").expect("write bounded fixture");

    assert_eq!(
        read_bounded_managed_file(&root, &path, 8).expect("read exact limit"),
        Some(b"12345678".to_vec())
    );
    let error =
        read_bounded_managed_file(&root, &path, 7).expect_err("larger managed file is rejected");
    assert!(matches!(
        error,
        BoundedManagedFileReadError::LimitExceeded {
            actual: 8,
            maximum: 7,
            ..
        }
    ));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn streamed_atomic_write_failure_preserves_existing_file() {
    let root = temp_dir("ic-query-confined-stream-failure");
    let path = root.join("nns/ic/archive/manifest.json");
    write_managed_text_atomically(&root, &path, "complete").expect("initial managed file");

    let error = write_managed_file_atomically(&root, &path, |file| {
        file.write_all(b"partial")?;
        Err(io::Error::other("fixture write failure"))
    })
    .expect_err("streamed replacement failure");

    assert!(matches!(error, CacheFileError::WriteTemp { .. }));
    assert_eq!(
        read_managed_text(&root, &path).expect("preserved managed file"),
        Some("complete".to_string())
    );
    assert_eq!(
        fs::read_dir(path.parent().expect("manifest parent"))
            .expect("manifest directory")
            .count(),
        1,
        "failed temporary file is removed"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn managed_path_rejects_escape() {
    let root = temp_dir("ic-query-confined-escape");
    let path = root.join("../outside.json");

    let error = write_managed_text_atomically(&root, &path, "evidence")
        .expect_err("parent traversal rejected");

    assert!(matches!(error, CacheFileError::Confinement { .. }));

    let outside = temp_dir("ic-query-confined-absolute-outside").join("cache.json");
    let outside_error = write_managed_text_atomically(&root, &outside, "evidence")
        .expect_err("absolute outside path rejected");
    assert!(matches!(outside_error, CacheFileError::Confinement { .. }));
    assert!(!outside.exists());
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn managed_path_rejects_symlinked_parent_and_file() {
    let root = temp_dir("ic-query-confined-symlink");
    let outside = temp_dir("ic-query-confined-outside");
    fs::create_dir_all(&outside).expect("create outside");
    fs::set_permissions(&outside, fs::Permissions::from_mode(0o700)).expect("secure outside");
    write_managed_text_atomically(&root, &root.join("seed.json"), "seed").expect("create root");

    symlink(&outside, root.join("linked-dir")).expect("link directory");
    let parent_error = read_managed_text(&root, &root.join("linked-dir/cache.json"))
        .expect_err("symlink parent rejected");
    assert!(matches!(parent_error, CacheFileError::Confinement { .. }));

    fs::write(outside.join("target.json"), "outside").expect("write outside target");
    symlink(outside.join("target.json"), root.join("linked.json")).expect("link file");
    let file_error =
        read_managed_text(&root, &root.join("linked.json")).expect_err("symlink file rejected");
    assert!(matches!(file_error, CacheFileError::Confinement { .. }));

    let linked_root = temp_dir("ic-query-confined-root-link");
    symlink(&outside, &linked_root).expect("link cache root");
    let root_error = read_managed_text(&linked_root, &linked_root.join("target.json"))
        .expect_err("symlink root rejected");
    assert!(matches!(root_error, CacheFileError::Confinement { .. }));

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_file(linked_root);
    let _ = fs::remove_dir_all(outside);
}

#[cfg(unix)]
#[test]
fn managed_path_rejects_unsafe_directory_and_file_modes() {
    let root = temp_dir("ic-query-confined-mode");
    let directory = root.join("nns");
    let path = directory.join("cache.json");
    write_managed_text_atomically(&root, &path, "evidence").expect("write managed cache");

    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("widen file mode");
    let file_error = read_managed_text(&root, &path).expect_err("unsafe file mode rejected");
    assert!(matches!(
        file_error,
        CacheFileError::UnsafeManagedPermissions {
            actual_mode: 0o644,
            ..
        }
    ));

    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("restore file mode");
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o755))
        .expect("widen managed directory mode");
    let directory_error =
        managed_file_exists(&root, &path).expect_err("unsafe managed directory rejected");
    assert!(matches!(
        directory_error,
        CacheFileError::UnsafeManagedPermissions {
            actual_mode: 0o755,
            ..
        }
    ));

    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
        .expect("restore managed directory mode");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).expect("widen root mode");
    let root_error = managed_file_exists(&root, &path).expect_err("unsafe root mode rejected");
    assert!(matches!(
        root_error,
        CacheFileError::UnsafeManagedPermissions {
            actual_mode: 0o755,
            ..
        }
    ));

    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).expect("restore root mode");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn missing_root_and_file_are_observed_without_creation() {
    let root = temp_dir("ic-query-confined-missing");
    let path = root.join("cache.json");

    assert_eq!(read_managed_file(&root, &path).expect("missing read"), None);
    assert!(!Path::new(&root).exists());
}
