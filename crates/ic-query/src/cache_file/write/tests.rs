use super::{validate_output_path, write_text_output};
use crate::{cache_file::CacheFileError, test_support::temp_dir};
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::{fs, path::Path};

fn assert_alias(output: &Path, managed: &Path) {
    let error = write_text_output(output, "replacement", &[managed])
        .expect_err("managed aliases must be rejected");
    assert!(matches!(
        error,
        CacheFileError::OutputAliasesManagedPath { output_path, managed_path }
            if output_path == output && managed_path == managed
    ));
}

#[test]
fn output_rejects_missing_managed_paths_and_parent_traversal_aliases() {
    let root = temp_dir("ic-query-output-missing-alias");
    let managed = root.join("cache/catalog.json");
    assert_alias(&managed, &managed);
    assert_alias(&root.join("cache/../cache/./catalog.json"), &managed);
    assert!(!root.exists());
}

#[test]
fn output_compares_relative_and_absolute_paths() {
    let name = "ic-query-output-relative-path-test.json";
    let absolute = fs::canonicalize(".").expect("working directory").join(name);
    let error = validate_output_path(Path::new(name), &[&absolute])
        .expect_err("relative and absolute names alias");
    assert!(matches!(
        error,
        CacheFileError::OutputAliasesManagedPath { .. }
    ));
}

#[test]
fn distinct_output_replaces_export_and_preserves_managed_file() {
    let root = temp_dir("ic-query-output-distinct");
    fs::create_dir_all(&root).expect("directory");
    let managed = root.join("catalog.json");
    let output = root.join("exports/catalog.json");
    fs::write(&managed, "original").expect("managed fixture");
    write_text_output(&output, "long initial export", &[&managed]).expect("first export");
    write_text_output(&output, "short", &[&managed]).expect("replace export");
    assert_eq!(
        fs::read_to_string(&managed).expect("managed file"),
        "original"
    );
    assert_eq!(fs::read_to_string(&output).expect("export"), "short");
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn output_rejects_file_symlink_parent_symlink_and_hard_link_aliases() {
    let root = temp_dir("ic-query-output-link-aliases");
    fs::create_dir_all(root.join("cache")).expect("directory");
    let managed = root.join("cache/catalog.json");
    fs::write(&managed, "original").expect("managed fixture");
    let symlink_path = root.join("catalog-link.json");
    symlink("cache/catalog.json", &symlink_path).expect("file symlink");
    let parent_link = root.join("cache-link");
    symlink("cache", &parent_link).expect("parent symlink");
    let hard_link = root.join("catalog-hard-link.json");
    fs::hard_link(&managed, &hard_link).expect("hard link");
    for output in [symlink_path, parent_link.join("catalog.json"), hard_link] {
        assert_alias(&output, &managed);
        assert_eq!(
            fs::read_to_string(&managed).expect("managed file"),
            "original"
        );
    }
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(unix)]
#[test]
fn output_rejects_dangling_symlink_aliases_before_creating_managed_files() {
    let root = temp_dir("ic-query-output-dangling-alias");
    fs::create_dir_all(&root).expect("directory");
    let managed = root.join("cache/catalog.json");
    let output = root.join("catalog-link.json");
    symlink("cache/catalog.json", &output).expect("dangling file symlink");
    assert_alias(&output, &managed);
    let parent_link = root.join("cache-link");
    symlink("cache", &parent_link).expect("dangling parent symlink");
    assert_alias(&parent_link.join("catalog.json"), &managed);
    assert!(!managed.exists());
    assert!(!managed.parent().expect("cache directory").exists());
    fs::remove_dir_all(root).expect("cleanup");
}

#[cfg(feature = "nns-host")]
#[test]
fn json_refresh_rejects_output_aliases_for_cache_and_lock_including_dry_runs() {
    use super::{RefreshCacheWriteRequest, write_json_refresh_cache};
    use crate::cache_file::write_managed_text_atomically;

    for dry_run in [true, false] {
        let root = temp_dir("ic-query-json-refresh-output-alias");
        let cache_path = root.join("cache/report.json");
        let lock_path = root.join("cache/refresh.lock");
        write_managed_text_atomically(&root, &cache_path, "original").expect("cache fixture");
        for output in [&cache_path, &lock_path] {
            let error = write_json_refresh_cache(
                RefreshCacheWriteRequest {
                    cache_root: &root,
                    cache_path: &cache_path,
                    lock_path: &lock_path,
                    network: "ic",
                    now_unix_secs: 0,
                    lock_stale_after_seconds: 60,
                    dry_run,
                    output_path: Some(output),
                    report: &serde_json::json!({"value": "replacement"}),
                },
                |error| error,
                |_, _| panic!("fixture must serialize"),
            )
            .expect_err("output aliases managed path");
            assert!(matches!(
                error,
                CacheFileError::OutputAliasesManagedPath { .. }
            ));
            assert_eq!(
                fs::read_to_string(&cache_path).expect("preserved cache"),
                "original"
            );
            assert!(!lock_path.exists());
        }
        fs::remove_dir_all(root).expect("cleanup");
    }
}
