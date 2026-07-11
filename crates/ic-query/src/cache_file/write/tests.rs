use super::{path::target_directory, write_text_atomically};
use crate::{cache_file::CacheFileError, test_support::temp_dir};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

#[test]
fn atomic_write_rejects_parentless_target_path() {
    let err = write_text_atomically(Path::new("/"), "data").expect_err("invalid path");

    match err {
        CacheFileError::WriteTemp { path, source } => {
            assert_eq!(path, PathBuf::from("/"));
            assert_eq!(source.kind(), ErrorKind::InvalidInput);
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn relative_single_file_target_uses_current_directory() {
    assert_eq!(
        target_directory(Path::new("cache.json")).expect("relative file target has directory"),
        Path::new(".")
    );
}

#[test]
fn failed_atomic_replace_removes_its_temp_file() {
    let root = temp_dir("ic-query-atomic-replace-cleanup");
    let target = root.join("cache.json");
    fs::create_dir_all(&root).expect("create fixture root");
    fs::create_dir(&target).expect("create directory at target path");

    let err = write_text_atomically(&target, "data").expect_err("directory target rejects rename");

    assert!(matches!(
        err,
        CacheFileError::Replace {
            target_path,
            ..
        } if target_path == target
    ));
    assert!(target.is_dir());
    let temp_prefix = "cache.json.tmp.";
    let temp_files = fs::read_dir(&root)
        .expect("read fixture root")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(temp_prefix))
        .collect::<Vec<_>>();
    assert!(temp_files.is_empty(), "orphaned temp files: {temp_files:?}");

    let _ = fs::remove_dir_all(root);
}
