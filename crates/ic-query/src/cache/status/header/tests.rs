use super::*;
use std::io::Cursor;
#[cfg(unix)]
use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

#[test]
fn unmanaged_history_status_reads_only_the_header_prefix() {
    let transactions = format!("{}0", "0,".repeat(10_000));
    let cache = format!(
        r#"{{"schema_version":1,"collection_completed_at":"2026-08-03T00:00:00Z","completeness":{{"status":"api_exhausted"}},"transactions":[{transactions}]}}"#
    );
    let mut reader = BufReader::with_capacity(64, Cursor::new(cache.as_bytes()));

    let header = read_cache_header(
        &classify_cache_path(Path::new(
            "icrc/endpoint/ledger/account/transactions/full.json",
        )),
        &mut reader,
        u64::try_from(cache.len()).unwrap(),
        1_024,
    )
    .expect("history header");

    assert_eq!(header.schema_version, 1);
    assert_eq!(
        header.fields.collection_completed_at.as_deref(),
        Some("2026-08-03T00:00:00Z")
    );
    assert!(reader.get_ref().position() <= 1_024);
    assert!(cache.len() > 10_000);
}

#[test]
fn registered_cache_rejects_ambiguous_schema_version_fields() {
    let error = read_cache_header(
        &classify_cache_path(Path::new("nns/ic/subnet-catalog/catalog.json")),
        Cursor::new(br#"{"schema_version":1,"catalog_schema_version":1,"network":"ic"}"#),
        0,
        1_024,
    )
    .err()
    .expect("multiple current schema fields are ambiguous");

    assert_eq!(error.io_error_kind(), Some(std::io::ErrorKind::InvalidData));
}

#[test]
fn registered_cache_inspection_bounds_metadata_and_stream_growth() {
    let path = Path::new("nns/ic/subnet-catalog/catalog.json");
    let mut reader = Cursor::new(br#"{"schema_version":1}             "#);
    let error = read_cache_header(&classify_cache_path(path), &mut reader, 32, 20)
        .err()
        .expect("oversized metadata");
    assert_eq!(error.io_error_kind(), Some(std::io::ErrorKind::InvalidData));
    assert_eq!(reader.position(), 0);

    let error = read_cache_header(&classify_cache_path(path), &mut reader, 19, 20)
        .err()
        .expect("growth after metadata admission");
    assert_eq!(error.io_error_kind(), Some(std::io::ErrorKind::InvalidData));
    assert_eq!(reader.position(), 21);
}

#[test]
fn unmanaged_cache_inspection_bounds_values_before_payload() {
    let cache = format!(
        r#"{{"schema_version":1,"extra":"{}","transactions":[]}}"#,
        "x".repeat(1_024)
    );
    let mut reader = Cursor::new(cache.as_bytes());
    let error = read_cache_header(
        &classify_cache_path(Path::new("icrc/ic/account/transactions/full.json")),
        &mut reader,
        u64::try_from(cache.len()).unwrap(),
        64,
    )
    .err()
    .expect("header inspection budget is exhausted");
    assert_eq!(error.io_error_kind(), Some(std::io::ErrorKind::InvalidData));
    assert_eq!(reader.position(), 64);

    let cache = br#"{"schema_version":1,"network":"ic","data":{"rows":[unread payload"#;
    let mut reader = Cursor::new(cache);
    let header = read_cache_header(
        &classify_cache_path(Path::new("sns/ic/root/neurons/full.json")),
        &mut reader,
        cache.len() as u64,
        64,
    )
    .expect("nested snapshot payload is not part of generic header inspection");
    assert_eq!(header.schema_version, 1);
    assert!(reader.position() < cache.len() as u64);
}

#[test]
fn registered_cache_inspection_checks_the_payload_syntax() {
    let cache = br#"{"schema_version":1,"subnets":[invalid]}"#;
    assert!(
        read_cache_header(
            &classify_cache_path(Path::new("nns/ic/subnet-catalog/catalog.json")),
            Cursor::new(cache),
            u64::try_from(cache.len()).unwrap(),
            1_024,
        )
        .is_err()
    );
}

#[test]
fn path_components_do_not_expose_variable_cache_identity() {
    for (path, expected) in [
        ("nns/ic/node/nodes.json", "nns/nodes"),
        (
            "nns/ic/governance/proposals/full.json",
            "nns/governance/proposals",
        ),
        ("sns/ic/root-principal/neurons/full.json", "sns/neurons"),
        (
            "sns/ic/catalog/discovery/full.json",
            "sns/catalog/discovery",
        ),
        (
            "icrc/ic/account-hash/transactions/full.json",
            "icrc/transactions",
        ),
        (
            "ic/ic/nodes/operational-status/full.json",
            "ic/nodes/operational-status",
        ),
    ] {
        assert_eq!(component_from_path(Path::new(path)), expected);
    }
}

#[cfg(unix)]
#[test]
fn opaque_path_components_keep_unregistered_caches_unmanaged() {
    let canonical = Path::new("nns/ic/subnet-catalog/catalog.json");
    let components = canonical.components().collect::<Vec<_>>();
    for position in 0..components.len() {
        let mut path = std::path::PathBuf::new();
        for (index, component) in components.iter().enumerate() {
            if index == position {
                path.push(OsStr::from_bytes(&[0xff]));
            }
            path.push(component.as_os_str());
        }
        let info = classify_cache_path(&path);
        assert_eq!(info.relative, path);
        assert_eq!(info.component, "unknown");
        assert_eq!(info.recovery_policy, CacheRecoveryPolicy::Unknown);
        assert_eq!(info.stale_after_seconds, None);
        assert!(matches!(info.inspection, CacheInspection::HeaderPrefix));
    }
}

#[test]
fn recovery_policy_follows_only_current_canonical_cache_paths() {
    for (path, expected) in [
        (
            "nns/ic/subnet-catalog/catalog.json",
            CacheRecoveryPolicy::Automatic,
        ),
        ("nns/ic/node/nodes.json", CacheRecoveryPolicy::Automatic),
        (
            "nns/ic/node-provider/providers.json",
            CacheRecoveryPolicy::Automatic,
        ),
        (
            "nns/ic/node-operator/operators.json",
            CacheRecoveryPolicy::Automatic,
        ),
        (
            "nns/ic/data-center/data-centers.json",
            CacheRecoveryPolicy::Automatic,
        ),
        (
            "sns/ic/catalog/discovery/full.json",
            CacheRecoveryPolicy::Automatic,
        ),
        (
            "ic/ic/nodes/operational-status/full.json",
            CacheRecoveryPolicy::Automatic,
        ),
        (
            "nns/ic/subnet-topology/report.json",
            CacheRecoveryPolicy::Explicit,
        ),
        (
            "nns/ic/governance/proposals/full.json",
            CacheRecoveryPolicy::Explicit,
        ),
        (
            "nns/ic/governance/neurons/full.json",
            CacheRecoveryPolicy::Explicit,
        ),
        (
            "sns/ic/root/neurons/full.json",
            CacheRecoveryPolicy::Explicit,
        ),
        (
            "icrc/ic/account/transactions/full.json",
            CacheRecoveryPolicy::Explicit,
        ),
        (
            "sns/ic/root/proposals/full.json",
            CacheRecoveryPolicy::MissingOnly,
        ),
        ("nns/local/node/nodes.json", CacheRecoveryPolicy::Unknown),
        ("unmanaged/ic/full.json", CacheRecoveryPolicy::Unknown),
    ] {
        assert_eq!(
            classify_cache_path(Path::new(path)).recovery_policy,
            expected,
            "{path}"
        );
    }
}

#[test]
fn age_policy_follows_only_current_canonical_mainnet_paths() {
    for (path, expected) in [
        (
            "nns/ic/subnet-catalog/catalog.json",
            Some(DEFAULT_STALE_AFTER_SECONDS),
        ),
        (
            "nns/ic/subnet-topology/report.json",
            Some(DEFAULT_NNS_SUBNET_TOPOLOGY_STALE_AFTER_SECONDS),
        ),
        (
            "sns/ic/catalog/discovery/full.json",
            Some(DEFAULT_SNS_CATALOG_STALE_AFTER_SECONDS),
        ),
        (
            "ic/ic/nodes/operational-status/full.json",
            Some(DEFAULT_IC_NODE_STATUS_STALE_AFTER_SECONDS),
        ),
        ("nns/local/subnet-catalog/catalog.json", None),
        ("unmanaged/ic/full.json", None),
    ] {
        assert_eq!(
            classify_cache_path(Path::new(path)).stale_after_seconds,
            expected,
            "{path}"
        );
    }
}
