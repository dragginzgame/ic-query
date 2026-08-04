use super::{fixtures::*, *};
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
    assert_eq!(report.sns_count, 1);
    assert!(!report.verbose);
    assert_eq!(report.sort, "id");
    assert_eq!(report.sns_instances[0].id, 1);
    assert_eq!(report.sns_instances[0].name, "Fixture SNS");
    assert_eq!(report.sns_instances[0].root_canister_id, ROOT_A);
    assert_eq!(report.metadata_error_count, 0);
    assert_eq!(report.data_source.as_str(), "live");
    assert_eq!(report.cache_path, None);
    assert_eq!(report.sns_instances[0].metadata_error, None);
    assert!(text.contains("ID   NAME"));
    assert!(text.contains("sort: id"));
    assert!(text.contains("metadata_errors: 0"));
    assert!(text.contains("Fixture SNS"));
    assert!(text.contains(&ROOT_A[..COMPACT_PRINCIPAL_CHARS]));
    assert!(!text.contains(ROOT_A));
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
    assert_eq!(source.inventory_calls.get(), 1);
    assert_eq!(source.metadata_calls.get(), 1);

    let mut stale_request = request;
    stale_request.now_unix_secs += DEFAULT_SNS_CATALOG_STALE_AFTER_SECONDS + 1;
    build_sns_list_report_from_cache_or_refresh_with_source(
        &stale_request,
        &root,
        &source,
        &mut progress,
    )
    .expect("stale catalog refresh");

    assert_eq!(source.inventory_calls.get(), 2);
    assert_eq!(source.metadata_calls.get(), 2);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn invalid_sns_catalog_is_not_silently_refreshed() {
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
    fs::write(sns_catalog_cache_path(&root, MAINNET_NETWORK), "not-json").expect("corrupt catalog");

    let error = build_sns_list_report_from_cache_or_refresh_with_source(
        &request,
        &root,
        &source,
        &mut progress,
    )
    .expect_err("invalid cache remains visible");

    assert!(matches!(error, SnsHostError::ParseCache { .. }));
    assert_eq!(source.inventory_calls.get(), 1);
    assert_eq!(source.metadata_calls.get(), 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn future_sns_catalog_is_not_silently_refreshed() {
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

    let error = build_sns_list_report_from_cache_or_refresh_with_source(
        &request,
        &root,
        &source,
        &mut progress,
    )
    .expect_err("future cache remains visible");

    assert!(
        matches!(error, SnsHostError::InvalidCache { reason, .. } if reason.contains("future"))
    );
    assert_eq!(source.inventory_calls.get(), 1);
    assert_eq!(source.metadata_calls.get(), 1);
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
    let report =
        build_sns_list_report_with_source(&list_request(false), &UnsortedFixtureSnsDiscoverySource)
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
    assert!(text.contains("metadata_error_details:"));
    assert!(text.contains("get_metadata: Canister has no Wasm module"));
    assert_eq!(
        info.metadata_error.as_deref(),
        Some("get_metadata: Canister has no Wasm module")
    );
    assert!(info_text.contains("metadata_error: get_metadata: Canister has no Wasm module"));
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

    assert_eq!(report.sns_count, 2);
    assert_eq!(
        source.metadata_targets.borrow().as_slice(),
        &[report
            .sns_instances
            .iter()
            .map(|sns| sns.root_canister_id.clone())
            .collect::<Vec<_>>()]
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

    assert!(matches!(
        inventory_error,
        SnsHostError::UnsupportedNetwork { network } if network == "local"
    ));
    assert!(matches!(
        metadata_error,
        SnsHostError::UnsupportedNetwork { network } if network == "local"
    ));
}

#[test]
fn sns_list_rejects_local_network() {
    let request = SnsListRequest {
        network: "local".to_string(),
        source_endpoint: DEFAULT_SNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_780_531_200,
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
    inventory_calls: Cell<usize>,
    metadata_calls: Cell<usize>,
}

impl SnsDiscoverySource for CountingDiscoverySource {
    fn fetch_sns_inventory(
        &self,
        request: &SnsSourceRequest,
    ) -> Result<MainnetSnsInventory, SnsHostError> {
        self.inventory_calls.set(self.inventory_calls.get() + 1);
        FixtureSnsDiscoverySource.fetch_sns_inventory(request)
    }

    fn fetch_sns_metadata(
        &self,
        request: &SnsSourceRequest,
        targets: &[MainnetSnsCanisters],
    ) -> Result<Vec<MainnetSnsMetadata>, SnsHostError> {
        self.metadata_calls.set(self.metadata_calls.get() + 1);
        FixtureSnsDiscoverySource.fetch_sns_metadata(request, targets)
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

#[derive(Default)]
struct RecordingDiscoverySource {
    metadata_targets: RefCell<Vec<Vec<String>>>,
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
