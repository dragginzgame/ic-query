use super::*;
use crate::{cache_file::create_managed_parent_directory, tests::temp_dir};
use std::fs;
#[cfg(all(feature = "sns-host", unix))]
use std::os::unix::fs::{PermissionsExt, symlink};

#[cfg(feature = "sns-host")]
#[test]
fn collection_discovery_requires_a_complete_candidate_and_entry_scan() {
    let root = temp_dir("ic-query-collection-scan-limit");
    let network = root.join("sns/ic");
    for entity in ["b", "a"] {
        super::super::write_managed_text_atomically(
            &root,
            &network.join(entity).join("proposals/full.json"),
            "{}",
        )
        .unwrap();
    }
    let paths =
        collect_managed_collection_files(&root, &network, "proposals", "full.json", 2).unwrap();
    assert_eq!(
        paths,
        vec![
            network.join("a/proposals/full.json"),
            network.join("b/proposals/full.json")
        ]
    );
    super::super::write_managed_text_atomically(
        &root,
        &network.join("c/proposals/full.json"),
        "{}",
    )
    .unwrap();
    assert!(matches!(
        collect_managed_collection_files(&root, &network, "proposals", "full.json", 2),
        Err(CacheFileError::ScanLimitExceeded {
            resource: "collection candidates",
            maximum: 2,
            ..
        })
    ));
    // Nonmatching entities must consume the entry allowance too.
    for entity in 0..14 {
        create_managed_parent_directory(&root, &network.join(entity.to_string()).join("probe"))
            .unwrap();
    }
    assert!(matches!(
        collect_managed_collection_files(&root, &network, "neurons", "full.json", 1),
        Err(CacheFileError::ScanLimitExceeded {
            resource: "directory entries",
            maximum: 16,
            ..
        })
    ));
    fs::remove_dir_all(root).unwrap();
}

#[cfg(all(feature = "sns-host", unix))]
#[test]
fn collection_discovery_preserves_confinement_and_permission_errors() {
    let root = temp_dir("ic-query-collection-scan-authority");
    let network = root.join("sns/ic");
    let path = network.join("entity/proposals/full.json");
    super::super::write_managed_text_atomically(&root, &path, "{}").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o664)).unwrap();
    assert!(matches!(
        collect_managed_collection_files(&root, &network, "proposals", "full.json", 2),
        Err(CacheFileError::UnsafeManagedPermissions { .. })
    ));
    fs::remove_file(&path).unwrap();
    symlink(&path, network.join("linked")).unwrap();
    assert!(matches!(
        collect_managed_collection_files(&root, &network, "proposals", "full.json", 2),
        Err(CacheFileError::Confinement { .. })
    ));
    fs::remove_dir_all(root).unwrap();
}

#[cfg(all(unix, feature = "sns-host"))]
#[test]
fn zero_budget_scans_project_unadmitted_entries_without_following_them() {
    let root = temp_dir("ic-query-zero-scan-budget");
    let outside = temp_dir("ic-query-zero-scan-outside");
    symlink(&outside, root.join("link")).unwrap();
    let inventory = collect_managed_files(&root, 0, |_| true).unwrap();
    assert!(inventory.root_found);
    assert!(inventory.truncated);
    assert_eq!(inventory.paths, Vec::<PathBuf>::new());
    assert!(matches!(
        collect_managed_collection_files(&root, &root, "proposals", "full.json", 0),
        Err(CacheFileError::ScanLimitExceeded {
            resource: "directory entries",
            maximum: 0,
            ..
        })
    ));
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[test]
fn inventory_bounds_nonmatching_entries_and_directory_depth() {
    let root = temp_dir("ic-query-inventory-entry-bound");
    for index in 0..17 {
        create_managed_parent_directory(&root, &root.join(index.to_string()).join("probe"))
            .unwrap();
    }
    let scan = collect_managed_files(&root, 1, |_| false).unwrap();
    assert!(scan.truncated);
    assert_eq!(scan.paths, Vec::<PathBuf>::new());
    fs::remove_dir_all(&root).unwrap();
    let path = (0..34).fold(root.clone(), |path, _| path.join("nested"));
    create_managed_parent_directory(&root, &path.join("probe")).unwrap();
    let scan = collect_managed_files(&root, 100, |_| false).unwrap();
    assert!(scan.truncated);
    fs::remove_dir_all(root).unwrap();
}
