use super::{path::target_directory, write_text_atomically};
use crate::cache_file::CacheFileError;
use std::{
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
