use super::*;
use crate::tests::temp_dir;
use std::fs;

#[test]
fn publication_preserves_distinct_snapshot_and_attempt_failures() {
    for snapshot_fails in [false, true] {
        let root = temp_dir("ic-query-governance-publication");
        let request = NnsGovernanceRefreshRequest::new(
            &root,
            "ic",
            "https://replica.example",
            1_700_000_000,
            25,
        );
        let paths = SnapshotJsonPaths {
            snapshot_path: root.join("snapshot.json"),
            refresh_lock_path: root.join("refresh.lock"),
            refresh_attempt_path: root.join("attempt.json"),
        };
        if snapshot_fails {
            fs::create_dir(&paths.snapshot_path).unwrap();
            fs::write(&paths.refresh_attempt_path, b"retained attempt evidence").unwrap();
        } else {
            fs::create_dir(&paths.refresh_attempt_path).unwrap();
        }
        let context = GovernanceSnapshotRefreshContext {
            request: &request,
            paths: &paths,
            component: "NNS publication fixture",
            replaced_existing_cache: false,
        };
        let result = context.publish::<_, HostCacheError>(
            crate::nns::NnsGovernanceCollection::Proposals,
            "fixture",
            SnapshotRefreshProgress::new(2, 3, None),
            serde_json::json!({"proposals": [1, 2, 3]}),
        );
        if snapshot_fails {
            assert!(matches!(result, Err(HostCacheError::Operation { .. })));
            assert_eq!(
                fs::read(&paths.refresh_attempt_path).unwrap(),
                b"retained attempt evidence"
            );
        } else {
            let report = result.unwrap();
            assert!(report.wrote_cache);
            assert!(report.attempt_finalization_error.is_some());
            let snapshot: serde_json::Value =
                serde_json::from_slice(&fs::read(&paths.snapshot_path).unwrap()).unwrap();
            assert_eq!(snapshot["schema_version"], 1);
            assert_eq!(snapshot["collection"], "proposals");
            assert_eq!(snapshot["completeness"]["row_count"], report.row_count);
            assert_eq!(snapshot["source_endpoint"], "https://replica.example");
            assert_eq!(snapshot["fetched_by"], "fixture");
            assert!(paths.refresh_attempt_path.is_dir());
        }
        fs::remove_dir_all(root).unwrap();
    }
}
