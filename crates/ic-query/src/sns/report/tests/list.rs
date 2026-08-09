use super::{fixtures::*, *};
use crate::QueryProgressEvent;
use std::{
    cell::{Cell, RefCell},
    fs,
    path::PathBuf,
    time::SystemTime,
};

#[test]
fn sns_list_report_uses_names_and_compact_ids_by_default() {
    let report =
        build_sns_list_report_with_source(&list_request(false), &FixtureSnsDiscoverySource)
            .expect("sns list report");
    let text = sns_list_report_text(&report);

    assert_eq!(report.schema_version, SNS_LIST_REPORT_SCHEMA_VERSION);
    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(report.sns_wasm_canister_id, MAINNET_SNS_WASM_CANISTER_ID);
    assert!(!report.all_lifecycles);
    assert_eq!(report.catalog_sns_count, 1);
    assert_eq!(report.excluded_sns_count, 0);
    assert_eq!(report.sns_count, 1);
    assert!(!report.verbose);
    assert_eq!(report.sort, "id");
    assert_eq!(report.sns_instances[0].id, 1);
    assert_eq!(report.sns_instances[0].name, "Fixture SNS");
    assert_eq!(report.sns_instances[0].root_canister_id, ROOT_A);
    assert_eq!(report.metadata_error_count, 0);
    assert_eq!(report.lifecycle_error_count, 0);
    assert_eq!(report.sns_instances[0].lifecycle, Some(3));
    assert_eq!(
        report.sns_instances[0].lifecycle_name.as_deref(),
        Some("committed")
    );
    assert_eq!(report.data_source.as_str(), "live");
    assert_eq!(report.cache_path, None);
    assert_eq!(report.sns_instances[0].metadata_error, None);
    assert!(text.contains("ID   NAME"));
    assert!(text.contains("METADATA"));
    assert!(text.contains("LIFECYCLE"));
    assert!(text.contains("all_lifecycles: no"));
    assert!(text.contains("sort: id"));
    assert!(text.contains("metadata_errors: 0"));
    assert!(text.contains("lifecycle_errors: 0"));
    assert!(text.contains("committed"));
    assert!(text.contains("ok"));
    assert!(text.contains("Fixture SNS"));
    assert!(text.contains(&ROOT_A[..COMPACT_PRINCIPAL_CHARS]));
    assert!(!text.contains(ROOT_A));

    let json = serde_json::to_value(&report).expect("serialize SNS list report");
    assert_eq!(json["sns_instances"][0]["lifecycle"], 3);
    assert_eq!(json["sns_instances"][0]["lifecycle_name"], "committed");
}

#[test]
fn sns_list_catalog_refreshes_missing_and_stale_but_reuses_fresh_cache() {
    let root = temp_catalog_root("ic-query-sns-catalog-policy");
    let source = CountingDiscoverySource::default();
    let mut progress = crate::progress::IgnoreQueryProgress;
    let request = list_request(false);

    let first = build_sns_list_report_from_cache_or_refresh_with_source(
        &request,
        &root,
        &source,
        &mut progress,
    )
    .expect("missing catalog refresh");
    let second = build_sns_list_report_from_cache_or_refresh_with_source(
        &request,
        &root,
        &source,
        &mut progress,
    )
    .expect("fresh catalog read");

    assert_eq!(first.data_source.as_str(), "cache");
    assert_eq!(first.cache_complete, Some(true));
    assert!(sns_list_report_text(&first).contains("cache_complete: yes"));
    assert_eq!(second.data_source.as_str(), "cache");
    assert_eq!(source.inventory.get(), 1);
    assert_eq!(source.metadata.get(), 1);
    assert_eq!(source.lifecycles.get(), 1);

    let mut stale_request = request;
    stale_request.now_unix_secs += DEFAULT_SNS_CATALOG_STALE_AFTER_SECONDS + 1;
    build_sns_list_report_from_cache_or_refresh_with_source(
        &stale_request,
        &root,
        &source,
        &mut progress,
    )
    .expect("stale catalog refresh");

    assert_eq!(source.inventory.get(), 2);
    assert_eq!(source.metadata.get(), 2);
    assert_eq!(source.lifecycles.get(), 2);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_list_refreshes_invalid_catalog_but_cache_only_remains_strict() {
    let root = temp_catalog_root("ic-query-invalid-sns-catalog");
    let source = CountingDiscoverySource::default();
    let mut progress = crate::progress::IgnoreQueryProgress;
    let request = list_request(false);
    build_sns_list_report_from_cache_or_refresh_with_source(
        &request,
        &root,
        &source,
        &mut progress,
    )
    .expect("initial catalog");
    let path = sns_catalog_cache_path(&root, MAINNET_NETWORK);
    fs::write(&path, "not-json").expect("corrupt catalog");

    let cache_only_error = build_sns_list_report_from_cache(&request, &root)
        .expect_err("cache-only read preserves invalid evidence");
    assert!(matches!(
        cache_only_error,
        SnsHostError::Cache(crate::HostCacheError::ParseCache { .. })
    ));

    let mut refresh_events = Vec::new();
    let mut recording_progress = |event| refresh_events.push(event);
    let report = build_sns_list_report_from_cache_or_refresh_with_source(
        &request,
        &root,
        &source,
        &mut recording_progress,
    )
    .expect("invalid cache refreshes");

    assert!(matches!(
        refresh_events.as_slice(),
        [QueryProgressEvent::CacheRefresh {
            component,
            path: refresh_path,
            source_endpoint,
        }] if component == "SNS catalog"
            && refresh_path == &path
            && source_endpoint == &request.source_endpoint
    ));
    assert_eq!(report.data_source.as_str(), "cache");
    assert_eq!(source.inventory.get(), 2);
    assert_eq!(source.metadata.get(), 2);
    assert_eq!(source.lifecycles.get(), 2);
    assert_ne!(
        fs::read_to_string(path).expect("read refreshed cache"),
        "not-json"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_list_refreshes_incompatible_catalog_headers() {
    for (case, mutate) in [
        (
            "schema",
            (|cache: &mut serde_json::Value| {
                cache["schema_version"] = serde_json::json!(999);
            }) as fn(&mut serde_json::Value),
        ),
        ("network", |cache: &mut serde_json::Value| {
            cache["network"] = serde_json::json!("local");
        }),
        ("identity", |cache: &mut serde_json::Value| {
            cache["domain"] = serde_json::json!("wrong");
        }),
    ] {
        let root = temp_catalog_root(&format!("ic-query-incompatible-sns-catalog-{case}"));
        let source = CountingDiscoverySource::default();
        let mut progress = crate::progress::IgnoreQueryProgress;
        let request = list_request(false);
        build_sns_list_report_from_cache_or_refresh_with_source(
            &request,
            &root,
            &source,
            &mut progress,
        )
        .expect("initial catalog");

        let path = sns_catalog_cache_path(&root, MAINNET_NETWORK);
        let mut cache =
            serde_json::from_slice::<serde_json::Value>(&fs::read(&path).expect("read catalog"))
                .expect("parse catalog");
        mutate(&mut cache);
        fs::write(
            &path,
            serde_json::to_vec_pretty(&cache).expect("serialize catalog"),
        )
        .expect("write incompatible catalog");

        build_sns_list_report_from_cache_or_refresh_with_source(
            &request,
            &root,
            &source,
            &mut progress,
        )
        .expect("incompatible catalog refreshes");
        assert_eq!(source.inventory.get(), 2);
        assert_eq!(source.metadata.get(), 2);
        assert_eq!(source.lifecycles.get(), 2);
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn sns_list_refreshes_future_dated_catalog() {
    let root = temp_catalog_root("ic-query-future-sns-catalog");
    let source = CountingDiscoverySource::default();
    let mut progress = crate::progress::IgnoreQueryProgress;
    let request = list_request(false);
    build_sns_list_report_from_cache_or_refresh_with_source(
        &request,
        &root,
        &source,
        &mut progress,
    )
    .expect("initial catalog");

    let path = sns_catalog_cache_path(&root, MAINNET_NETWORK);
    let mut cache = serde_json::from_str::<serde_json::Value>(
        &fs::read_to_string(&path).expect("read catalog"),
    )
    .expect("parse catalog");
    cache["fetched_at"] = serde_json::Value::String("9999-01-01T00:00:00Z".to_string());
    fs::write(
        &path,
        serde_json::to_string_pretty(&cache).expect("serialize catalog"),
    )
    .expect("write future catalog");

    let report = build_sns_list_report_from_cache_or_refresh_with_source(
        &request,
        &root,
        &source,
        &mut progress,
    )
    .expect("future cache refreshes");

    assert_eq!(
        report.fetched_at,
        format_utc_timestamp_secs(request.now_unix_secs)
    );
    assert_eq!(source.inventory.get(), 2);
    assert_eq!(source.metadata.get(), 2);
    assert_eq!(source.lifecycles.get(), 2);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn failed_invalid_catalog_refresh_preserves_original_file() {
    let root = temp_catalog_root("ic-query-failed-invalid-sns-catalog-refresh");
    let mut progress = crate::progress::IgnoreQueryProgress;
    let request = list_request(false);
    build_sns_list_report_from_cache_or_refresh_with_source(
        &request,
        &root,
        &CountingDiscoverySource::default(),
        &mut progress,
    )
    .expect("initial catalog");

    let path = sns_catalog_cache_path(&root, MAINNET_NETWORK);
    fs::write(&path, "not-json").expect("corrupt catalog");
    let error = build_sns_list_report_from_cache_or_refresh_with_source(
        &request,
        &root,
        &MutatingInventorySource(invalid_inventory_root),
        &mut progress,
    )
    .expect_err("failed refresh remains visible");

    assert!(matches!(error, SnsHostError::InvalidSourceData { .. }));
    assert_eq!(
        fs::read_to_string(path).expect("read preserved invalid cache"),
        "not-json"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn sns_list_report_rejects_custom_inventory_provenance_and_identity_failures() {
    for (mutate, expected_reason) in [
        (
            wrong_inventory_endpoint as fn(&mut MainnetSnsInventory),
            "source_endpoint",
        ),
        (invalid_inventory_root, "root_canister_id"),
        (duplicate_inventory_root, "duplicate root canister id"),
    ] {
        let error = build_sns_list_report_with_source(
            &list_request(false),
            &MutatingInventorySource(mutate),
        )
        .expect_err("invalid custom SNS inventory must fail");

        assert!(matches!(
            error,
            SnsHostError::InvalidSourceData {
                capability: "SNS-W deployed SNS inventory",
                reason,
            } if reason.contains(expected_reason)
        ));
    }
}

#[test]
fn sns_list_report_rejects_inexact_metadata_results() {
    for (mutate, expected_reason) in [
        (
            remove_metadata as fn(&mut Vec<MainnetSnsMetadata>),
            "missing requested",
        ),
        (duplicate_metadata, "duplicate metadata root"),
        (return_unrequested_metadata, "unrequested root"),
        (invalidate_metadata_root, "is invalid"),
        (empty_metadata_error, "metadata_error"),
        (untrimmed_metadata_name, "surrounding whitespace"),
        (untrimmed_metadata_error, "surrounding whitespace"),
        (contradictory_metadata_result, "both payload fields"),
    ] {
        let error = build_sns_list_report_with_source(
            &list_request(false),
            &MutatingMetadataSource(mutate),
        )
        .expect_err("inexact custom SNS metadata must fail");

        assert!(matches!(
            error,
            SnsHostError::InvalidSourceData {
                capability: "SNS metadata",
                reason,
            } if reason.contains(expected_reason)
        ));
    }
}

#[test]
fn sns_list_report_rejects_inexact_lifecycle_results() {
    for (mutate, expected_reason) in [
        (
            remove_lifecycle as fn(&mut Vec<MainnetSnsLifecycle>),
            "missing requested",
        ),
        (duplicate_lifecycle, "duplicate lifecycle root"),
        (return_unrequested_lifecycle, "unrequested root"),
        (invalidate_lifecycle_root, "is invalid"),
        (empty_lifecycle_error, "lifecycle_error"),
        (untrimmed_lifecycle_error, "surrounding whitespace"),
        (contradictory_lifecycle_result, "both value fields"),
        (mismatched_lifecycle_name, "lifecycle_name"),
    ] {
        let error = build_sns_list_report_with_source(
            &list_request(false),
            &MutatingLifecycleSource(mutate),
        )
        .expect_err("inexact custom SNS lifecycle must fail");

        assert!(matches!(
            error,
            SnsHostError::InvalidSourceData {
                capability: "SNS lifecycle",
                reason,
            } if reason.contains(expected_reason)
        ));
    }
}

#[test]
fn sns_list_all_surfaces_lifecycle_query_errors_hidden_by_default() {
    let source = MutatingLifecycleSource(lifecycle_query_error);
    let default_report = build_sns_list_report_with_source(&list_request(false), &source)
        .expect("default list with lifecycle error");
    let all_report =
        build_sns_list_report_with_source(&list_request(false).with_all_lifecycles(true), &source)
            .expect("all list with lifecycle error");

    assert_eq!(default_report.catalog_sns_count, 1);
    assert_eq!(default_report.sns_count, 0);
    assert_eq!(default_report.excluded_sns_count, 1);
    assert_eq!(default_report.lifecycle_error_count, 0);
    assert_eq!(all_report.sns_count, 1);
    assert_eq!(all_report.lifecycle_error_count, 1);
    assert_eq!(
        all_report.sns_instances[0].lifecycle_error.as_deref(),
        Some("get_lifecycle: query rejected")
    );
    assert!(sns_list_report_text(&all_report).contains("error"));
}

#[test]
fn sns_list_report_verbose_text_keeps_full_ids() {
    let report = build_sns_list_report_with_source(&list_request(true), &FixtureSnsDiscoverySource)
        .expect("sns list report");
    let text = sns_list_report_text(&report);

    assert!(report.verbose);
    assert!(text.contains(ROOT_A));
    assert!(text.contains(GOVERNANCE_A));
}

#[test]
fn sns_info_resolves_list_id() {
    let report = build_sns_info_report_with_source(&info_request("1"), &FixtureSnsDiscoverySource)
        .expect("sns info report");
    let text = sns_info_report_text(&report);

    assert_eq!(report.schema_version, SNS_INFO_REPORT_SCHEMA_VERSION);
    assert_eq!(report.id, 1);
    assert_eq!(report.name, "Fixture SNS");
    assert_eq!(report.description.as_deref(), Some("Fixture description"));
    assert_eq!(report.url.as_deref(), Some("https://example.com"));
    assert_eq!(report.metadata_error, None);
    assert!(text.contains("root_canister_id: be2us-64aaa-aaaaa-qaabq-cai"));
}

#[test]
fn sns_info_resolves_root_principal() {
    let report =
        build_sns_info_report_with_source(&info_request(ROOT_A), &FixtureSnsDiscoverySource)
            .expect("sns info report");

    assert_eq!(report.id, 1);
    assert_eq!(report.root_canister_id, ROOT_A);
}

#[test]
fn sns_list_ids_follow_source_order() {
    let request = list_request(false).with_all_lifecycles(true);
    let report = build_sns_list_report_with_source(&request, &UnsortedFixtureSnsDiscoverySource)
        .expect("sns list report");
    let info =
        build_sns_info_report_with_source(&info_request("1"), &UnsortedFixtureSnsDiscoverySource)
            .expect("sns info report");

    assert_eq!(report.sns_instances[0].id, 1);
    assert_eq!(report.sns_instances[0].name, "A Name");
    assert_eq!(report.sns_instances[0].root_canister_id, ROOT_A);
    assert_eq!(report.sns_instances[1].id, 2);
    assert_eq!(report.sns_instances[1].name, "Z Name");
    assert_eq!(info.id, 1);
    assert_eq!(info.root_canister_id, ROOT_A);
}

#[test]
fn sns_list_name_sort_keeps_stable_ids() {
    let mut request = list_request(false);
    request.all_lifecycles = true;
    request.sort = SnsListSort::Name;

    let report = build_sns_list_report_with_source(&request, &UnsortedFixtureSnsDiscoverySource)
        .expect("sns list report");
    let text = sns_list_report_text(&report);
    let info =
        build_sns_info_report_with_source(&info_request("1"), &UnsortedFixtureSnsDiscoverySource)
            .expect("sns info report");

    assert_eq!(report.sort, "name");
    assert_eq!(report.sns_instances[0].id, 1);
    assert_eq!(report.sns_instances[0].name, "A Name");
    assert_eq!(report.sns_instances[1].id, 2);
    assert_eq!(report.sns_instances[1].name, "Z Name");
    assert!(text.contains("sort: name"));
    assert_eq!(info.id, 1);
    assert_eq!(info.root_canister_id, ROOT_A);
}

#[test]
fn sns_list_filters_to_committed_by_default_and_all_preserves_stable_ids() {
    let default_report =
        build_sns_list_report_with_source(&list_request(false), &UnsortedFixtureSnsDiscoverySource)
            .expect("default SNS list report");
    let all_report = build_sns_list_report_with_source(
        &list_request(false).with_all_lifecycles(true),
        &UnsortedFixtureSnsDiscoverySource,
    )
    .expect("all-lifecycle SNS list report");

    assert_eq!(default_report.catalog_sns_count, 2);
    assert_eq!(default_report.excluded_sns_count, 1);
    assert_eq!(default_report.sns_count, 1);
    assert_eq!(default_report.sns_instances[0].id, 1);
    assert_eq!(default_report.sns_instances[0].lifecycle, Some(3));
    assert_eq!(all_report.catalog_sns_count, 2);
    assert_eq!(all_report.excluded_sns_count, 0);
    assert_eq!(all_report.sns_count, 2);
    assert_eq!(all_report.sns_instances[1].id, 2);
    assert_eq!(all_report.sns_instances[1].root_canister_id, ROOT_B);
    assert_eq!(all_report.sns_instances[1].lifecycle, Some(4));
    assert_eq!(
        all_report.sns_instances[1].lifecycle_name.as_deref(),
        Some("aborted")
    );
    assert!(sns_list_report_text(&all_report).contains("aborted"));
}

#[test]
fn sns_list_surfaces_metadata_fallbacks() {
    let report = build_sns_list_report_with_source(
        &list_request(true),
        &MetadataErrorFixtureSnsDiscoverySource,
    )
    .expect("sns list report");
    let text = sns_list_report_text(&report);
    let info = build_sns_info_report_with_source(
        &info_request("1"),
        &MetadataErrorFixtureSnsDiscoverySource,
    )
    .expect("sns info report");
    let info_text = sns_info_report_text(&info);

    assert_eq!(report.metadata_error_count, 1);
    assert_eq!(report.sns_instances[0].name, "unnamed-be2us");
    assert_eq!(
        report.sns_instances[0].metadata_error.as_deref(),
        Some("get_metadata: Canister has no Wasm module")
    );
    assert!(text.contains("metadata_errors: 1"));
    assert!(text.contains("no_wasm"));
    assert!(!text.contains("metadata_error_details:"));
    assert!(!text.contains("get_metadata: Canister has no Wasm module"));
    assert_eq!(
        info.metadata_error.as_deref(),
        Some("get_metadata: Canister has no Wasm module")
    );
    assert!(info_text.contains("metadata_error: get_metadata: Canister has no Wasm module"));

    let mut other_error = report;
    other_error.sns_instances[0].metadata_error = Some("get_metadata: timed out".to_string());
    let other_error_text = sns_list_report_text(&other_error);
    assert!(
        other_error_text
            .lines()
            .any(|line| line.contains("unnamed-be2us") && line.ends_with("error"))
    );
    assert!(!other_error_text.contains("get_metadata: timed out"));
}

#[test]
fn direct_lookup_enriches_only_the_resolved_sns() {
    let source = RecordingDiscoverySource::default();

    let report = build_sns_info_report_with_source(&info_request("2"), &source)
        .expect("targeted SNS info report");

    assert_eq!(report.id, 2);
    assert_eq!(
        source.metadata_targets.borrow().as_slice(),
        &[vec![report.root_canister_id]]
    );
}

#[test]
fn list_enriches_the_complete_inventory() {
    let source = RecordingDiscoverySource::default();

    let report = build_sns_list_report_with_source(&list_request(false), &source)
        .expect("complete SNS list report");

    assert_eq!(report.catalog_sns_count, 2);
    assert_eq!(report.sns_count, 1);
    assert_eq!(
        source.metadata_targets.borrow().as_slice(),
        &[vec![ROOT_A.to_string(), ROOT_B.to_string()]]
    );
    assert_eq!(
        source.lifecycle_targets.borrow().as_slice(),
        &[vec![ROOT_A.to_string(), ROOT_B.to_string()]]
    );
}

#[test]
fn unknown_lookup_does_not_request_metadata() {
    let source = RecordingDiscoverySource::default();

    let error = build_sns_info_report_with_source(&info_request("3"), &source)
        .expect_err("unknown SNS id must fail before metadata");

    assert!(matches!(
        error,
        SnsHostError::UnknownSnsId {
            id: 3,
            sns_count: 2
        }
    ));
    assert!(source.metadata_targets.borrow().is_empty());
}

#[test]
fn live_sns_discovery_rejects_non_mainnet_before_agent_construction() {
    let request = SnsSourceRequest::new("local", "not a valid endpoint", "timestamp", "test");

    let inventory_error = LiveSnsSource
        .fetch_sns_inventory(&request)
        .expect_err("non-mainnet inventory must fail");
    let metadata_error = LiveSnsSource
        .fetch_sns_metadata(&request, &[fixture_canisters_a()])
        .expect_err("non-mainnet metadata must fail");
    let lifecycle_error = LiveSnsSource
        .fetch_sns_lifecycles(&request, &[fixture_canisters_a()])
        .expect_err("non-mainnet lifecycle must fail");

    assert!(matches!(
        inventory_error,
        SnsHostError::UnsupportedNetwork { network } if network == "local"
    ));
    assert!(matches!(
        metadata_error,
        SnsHostError::UnsupportedNetwork { network } if network == "local"
    ));
    assert!(matches!(
        lifecycle_error,
        SnsHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[test]
fn sns_list_rejects_local_network() {
    let request = SnsListRequest {
        network: "local".to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
        all_lifecycles: false,
        verbose: false,
        sort: SnsListSort::Id,
    };

    let err = build_sns_list_report_with_source(&request, &FixtureSnsDiscoverySource)
        .expect_err("local rejected");

    assert!(matches!(err, SnsHostError::UnsupportedNetwork { .. }));
}

struct MutatingInventorySource(fn(&mut MainnetSnsInventory));

#[derive(Default)]
struct CountingDiscoverySource {
    inventory: Cell<usize>,
    metadata: Cell<usize>,
    lifecycles: Cell<usize>,
}

impl SnsDiscoverySource for CountingDiscoverySource {
    fn fetch_sns_inventory(
        &self,
        request: &SnsSourceRequest,
    ) -> Result<MainnetSnsInventory, SnsHostError> {
        self.inventory.set(self.inventory.get() + 1);
        FixtureSnsDiscoverySource.fetch_sns_inventory(request)
    }

    fn fetch_sns_metadata(
        &self,
        request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
        self.metadata.set(self.metadata.get() + 1);
        FixtureSnsDiscoverySource.fetch_sns_metadata(request, targets)
    }
}

impl SnsCatalogSource for CountingDiscoverySource {
    fn fetch_sns_lifecycles(
        &self,
        request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsLifecycle>, SnsHostError> {
        self.lifecycles.set(self.lifecycles.get() + 1);
        FixtureSnsDiscoverySource.fetch_sns_lifecycles(request, targets)
    }
}

fn temp_catalog_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{nonce}"))
}

impl SnsDiscoverySource for MutatingInventorySource {
    fn fetch_sns_inventory(
        &self,
        request: &SnsSourceRequest,
    ) -> Result<MainnetSnsInventory, SnsHostError> {
        let mut inventory = FixtureSnsDiscoverySource.fetch_sns_inventory(request)?;
        self.0(&mut inventory);
        Ok(inventory)
    }

    fn fetch_sns_metadata(
        &self,
        request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
        FixtureSnsDiscoverySource.fetch_sns_metadata(request, targets)
    }
}

impl SnsCatalogSource for MutatingInventorySource {
    fn fetch_sns_lifecycles(
        &self,
        request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsLifecycle>, SnsHostError> {
        FixtureSnsDiscoverySource.fetch_sns_lifecycles(request, targets)
    }
}

struct MutatingMetadataSource(fn(&mut Vec<MainnetSnsMetadata>));

impl SnsDiscoverySource for MutatingMetadataSource {
    fn fetch_sns_inventory(
        &self,
        request: &SnsSourceRequest,
    ) -> Result<MainnetSnsInventory, SnsHostError> {
        FixtureSnsDiscoverySource.fetch_sns_inventory(request)
    }

    fn fetch_sns_metadata(
        &self,
        request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
        let mut metadata = FixtureSnsDiscoverySource.fetch_sns_metadata(request, targets)?;
        self.0(&mut metadata);
        Ok(metadata)
    }
}

impl SnsCatalogSource for MutatingMetadataSource {
    fn fetch_sns_lifecycles(
        &self,
        request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsLifecycle>, SnsHostError> {
        FixtureSnsDiscoverySource.fetch_sns_lifecycles(request, targets)
    }
}

struct MutatingLifecycleSource(fn(&mut Vec<MainnetSnsLifecycle>));

impl SnsDiscoverySource for MutatingLifecycleSource {
    fn fetch_sns_inventory(
        &self,
        request: &SnsSourceRequest,
    ) -> Result<MainnetSnsInventory, SnsHostError> {
        FixtureSnsDiscoverySource.fetch_sns_inventory(request)
    }

    fn fetch_sns_metadata(
        &self,
        request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
        FixtureSnsDiscoverySource.fetch_sns_metadata(request, targets)
    }
}

impl SnsCatalogSource for MutatingLifecycleSource {
    fn fetch_sns_lifecycles(
        &self,
        request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsLifecycle>, SnsHostError> {
        let mut lifecycles = FixtureSnsDiscoverySource.fetch_sns_lifecycles(request, targets)?;
        self.0(&mut lifecycles);
        Ok(lifecycles)
    }
}

#[derive(Default)]
struct RecordingDiscoverySource {
    metadata_targets: RefCell<Vec<Vec<String>>>,
    lifecycle_targets: RefCell<Vec<Vec<String>>>,
}

impl SnsDiscoverySource for RecordingDiscoverySource {
    fn fetch_sns_inventory(
        &self,
        request: &SnsSourceRequest,
    ) -> Result<MainnetSnsInventory, SnsHostError> {
        UnsortedFixtureSnsDiscoverySource.fetch_sns_inventory(request)
    }

    fn fetch_sns_metadata(
        &self,
        request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
        self.metadata_targets.borrow_mut().push(
            targets
                .iter()
                .map(|target| target.root_canister_id.clone())
                .collect(),
        );
        UnsortedFixtureSnsDiscoverySource.fetch_sns_metadata(request, targets)
    }
}

impl SnsCatalogSource for RecordingDiscoverySource {
    fn fetch_sns_lifecycles(
        &self,
        request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsLifecycle>, SnsHostError> {
        self.lifecycle_targets.borrow_mut().push(
            targets
                .iter()
                .map(|target| target.root_canister_id.clone())
                .collect(),
        );
        UnsortedFixtureSnsDiscoverySource.fetch_sns_lifecycles(request, targets)
    }
}

fn wrong_inventory_endpoint(inventory: &mut MainnetSnsInventory) {
    inventory.source_endpoint = "https://wrong.example".to_string();
}

fn invalid_inventory_root(inventory: &mut MainnetSnsInventory) {
    inventory.sns_instances[0].root_canister_id = "not a principal".to_string();
}

fn duplicate_inventory_root(inventory: &mut MainnetSnsInventory) {
    inventory
        .sns_instances
        .push(inventory.sns_instances[0].clone());
}

fn remove_metadata(metadata: &mut Vec<MainnetSnsMetadata>) {
    metadata.clear();
}

fn duplicate_metadata(metadata: &mut Vec<MainnetSnsMetadata>) {
    metadata.push(metadata[0].clone());
}

#[expect(clippy::ptr_arg, reason = "shared metadata mutation fixture signature")]
fn return_unrequested_metadata(metadata: &mut Vec<MainnetSnsMetadata>) {
    metadata[0].root_canister_id = GOVERNANCE_A.to_string();
}

#[expect(clippy::ptr_arg, reason = "shared metadata mutation fixture signature")]
fn invalidate_metadata_root(metadata: &mut Vec<MainnetSnsMetadata>) {
    metadata[0].root_canister_id = "not a principal".to_string();
}

#[expect(clippy::ptr_arg, reason = "shared metadata mutation fixture signature")]
fn empty_metadata_error(metadata: &mut Vec<MainnetSnsMetadata>) {
    metadata[0].name = None;
    metadata[0].description = None;
    metadata[0].url = None;
    metadata[0].metadata_error = Some("  ".to_string());
}

#[expect(clippy::ptr_arg, reason = "shared metadata mutation fixture signature")]
fn untrimmed_metadata_name(metadata: &mut Vec<MainnetSnsMetadata>) {
    metadata[0].name = Some(" Fixture SNS ".to_string());
}

#[expect(clippy::ptr_arg, reason = "shared metadata mutation fixture signature")]
fn untrimmed_metadata_error(metadata: &mut Vec<MainnetSnsMetadata>) {
    metadata[0].name = None;
    metadata[0].description = None;
    metadata[0].url = None;
    metadata[0].metadata_error = Some(" fixture failure ".to_string());
}

#[expect(clippy::ptr_arg, reason = "shared metadata mutation fixture signature")]
fn contradictory_metadata_result(metadata: &mut Vec<MainnetSnsMetadata>) {
    metadata[0].metadata_error = Some("fixture failure".to_string());
}

fn remove_lifecycle(lifecycles: &mut Vec<MainnetSnsLifecycle>) {
    lifecycles.clear();
}

fn duplicate_lifecycle(lifecycles: &mut Vec<MainnetSnsLifecycle>) {
    lifecycles.push(lifecycles[0].clone());
}

#[expect(
    clippy::ptr_arg,
    reason = "shared lifecycle mutation fixture signature"
)]
fn return_unrequested_lifecycle(lifecycles: &mut Vec<MainnetSnsLifecycle>) {
    lifecycles[0].root_canister_id = GOVERNANCE_A.to_string();
}

#[expect(
    clippy::ptr_arg,
    reason = "shared lifecycle mutation fixture signature"
)]
fn invalidate_lifecycle_root(lifecycles: &mut Vec<MainnetSnsLifecycle>) {
    lifecycles[0].root_canister_id = "not a principal".to_string();
}

#[expect(
    clippy::ptr_arg,
    reason = "shared lifecycle mutation fixture signature"
)]
fn empty_lifecycle_error(lifecycles: &mut Vec<MainnetSnsLifecycle>) {
    lifecycles[0].lifecycle = None;
    lifecycles[0].lifecycle_name = None;
    lifecycles[0].lifecycle_error = Some("  ".to_string());
}

#[expect(
    clippy::ptr_arg,
    reason = "shared lifecycle mutation fixture signature"
)]
fn untrimmed_lifecycle_error(lifecycles: &mut Vec<MainnetSnsLifecycle>) {
    lifecycles[0].lifecycle = None;
    lifecycles[0].lifecycle_name = None;
    lifecycles[0].lifecycle_error = Some(" query rejected ".to_string());
}

#[expect(
    clippy::ptr_arg,
    reason = "shared lifecycle mutation fixture signature"
)]
fn contradictory_lifecycle_result(lifecycles: &mut Vec<MainnetSnsLifecycle>) {
    lifecycles[0].lifecycle_error = Some("query rejected".to_string());
}

#[expect(
    clippy::ptr_arg,
    reason = "shared lifecycle mutation fixture signature"
)]
fn mismatched_lifecycle_name(lifecycles: &mut Vec<MainnetSnsLifecycle>) {
    lifecycles[0].lifecycle_name = Some("aborted".to_string());
}

#[expect(
    clippy::ptr_arg,
    reason = "shared lifecycle mutation fixture signature"
)]
fn lifecycle_query_error(lifecycles: &mut Vec<MainnetSnsLifecycle>) {
    lifecycles[0].lifecycle = None;
    lifecycles[0].lifecycle_name = None;
    lifecycles[0].lifecycle_error = Some("get_lifecycle: query rejected".to_string());
}
