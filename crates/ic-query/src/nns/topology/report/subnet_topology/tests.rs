use super::*;
use crate::{
    nns::{
        NnsSourceRequest,
        topology::report::subnet_topology::NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION,
    },
    subnet_catalog::{MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, SubnetKind},
    test_support::temp_dir,
};
use candid::Principal;
use std::{cell::Cell, fs, path::PathBuf};

const NOW: u64 = 1_700_000_000;

#[test]
fn report_validation_enforces_canonical_counts_and_order() {
    let report = fixture_report(42, "2023-11-14T22:13:20Z");
    report.validate().expect("valid report");

    let mut mismatched = report.clone();
    mismatched.subnets[0].node_count += 1;
    assert!(matches!(
        mismatched.validate().expect_err("count mismatch"),
        NnsSubnetTopologyValidationError::SubnetNodeCountMismatch { .. }
    ));

    let mut unordered = report;
    unordered.subnets.reverse();
    assert!(matches!(
        unordered.validate().expect_err("order mismatch"),
        NnsSubnetTopologyValidationError::NonCanonicalSubnetOrder { .. }
    ));
}

#[test]
fn refresh_holds_one_lock_and_publishes_one_validated_snapshot() {
    let root = temp_dir("ic-query-subnet-topology-refresh");
    let request = refresh_request(&root, NOW);
    let lock_path = nns_subnet_topology_refresh_lock_path(&root, MAINNET_NETWORK);
    let source = FixtureSource::new(42, Some(lock_path.clone()));

    let cached =
        refresh_nns_subnet_topology_with_source(&request, &source).expect("refresh snapshot");

    assert_eq!(source.calls.get(), 1);
    assert_eq!(cached.report.registry_version, 42);
    assert_eq!(
        cached.path,
        nns_subnet_topology_cache_path(&root, MAINNET_NETWORK)
    );
    assert!(!lock_path.exists());
    let loaded = load_cached_nns_subnet_topology(&request.cache).expect("load snapshot");
    assert_eq!(loaded.report, cached.report);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn missing_and_stale_refresh_policies_are_explicitly_distinct() {
    let root = temp_dir("ic-query-subnet-topology-policies");
    let old_request = refresh_request(&root, NOW);
    refresh_nns_subnet_topology_with_source(&old_request, &FixtureSource::new(1, None))
        .expect("initial refresh");

    let later_request = refresh_request(&root, NOW + 100);
    let missing_source = FixtureSource::new(2, None);
    let loaded =
        load_or_refresh_missing_nns_subnet_topology_with_source(&later_request, &missing_source)
            .expect("existing cache");
    assert_eq!(loaded.report.registry_version, 1);
    assert_eq!(missing_source.calls.get(), 0);

    let stale_source = FixtureSource::new(2, None);
    let refreshed =
        load_or_refresh_stale_nns_subnet_topology_with_source(&later_request, 10, &stale_source)
            .expect("stale refresh");
    assert_eq!(refreshed.report.registry_version, 2);
    assert_eq!(stale_source.calls.get(), 1);

    let fresh_source = FixtureSource::new(3, None);
    let fresh =
        load_or_refresh_stale_nns_subnet_topology_with_source(&later_request, 10, &fresh_source)
            .expect("fresh cache");
    assert_eq!(fresh.report.registry_version, 2);
    assert_eq!(fresh_source.calls.get(), 0);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn invalid_refresh_preserves_the_last_complete_snapshot() {
    let root = temp_dir("ic-query-subnet-topology-invalid-refresh");
    let request = refresh_request(&root, NOW);
    refresh_nns_subnet_topology_with_source(&request, &FixtureSource::new(1, None))
        .expect("initial refresh");

    let invalid_source = InvalidFixtureSource;
    let error = refresh_nns_subnet_topology_with_source(&request, &invalid_source)
        .expect_err("invalid report");
    assert!(matches!(
        error,
        NnsSubnetTopologyHostError::Validation(
            NnsSubnetTopologyValidationError::ZeroNodeProviderCount { .. }
        )
    ));

    let loaded = load_cached_nns_subnet_topology(&request.cache).expect("old snapshot");
    assert_eq!(loaded.report.registry_version, 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn cache_load_is_local_only_and_missing_is_typed() {
    let root = temp_dir("ic-query-subnet-topology-missing");
    let request = NnsSubnetTopologyCacheRequest::new(&root, MAINNET_NETWORK);

    let error = load_cached_nns_subnet_topology(&request).expect_err("missing cache");

    assert!(matches!(
        error,
        NnsSubnetTopologyHostError::Cache(crate::HostCacheError::MissingCache { path, .. })
            if path == nns_subnet_topology_cache_path(&root, MAINNET_NETWORK)
    ));
}

#[test]
fn text_preserves_raw_subnet_kind_and_provider_counts() {
    let report = fixture_report(42, "2023-11-14T22:13:20Z");

    let text = nns_subnet_topology_report_text(&report);

    assert!(text.contains("cloud_engine"));
    assert!(text.contains("PROVIDER_NODES"));
    assert!(text.contains("registry_version 42"));
}

struct FixtureSource {
    registry_version: u64,
    expected_lock_path: Option<PathBuf>,
    calls: Cell<usize>,
}

impl FixtureSource {
    const fn new(registry_version: u64, expected_lock_path: Option<PathBuf>) -> Self {
        Self {
            registry_version,
            expected_lock_path,
            calls: Cell::new(0),
        }
    }
}

impl NnsSubnetTopologySource for FixtureSource {
    fn fetch_subnet_topology_report(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
        self.calls.set(self.calls.get() + 1);
        if let Some(lock_path) = &self.expected_lock_path {
            assert!(
                lock_path.is_file(),
                "source fetch must run under the cache lock"
            );
        }
        Ok(fixture_report(self.registry_version, &request.fetched_at))
    }
}

struct InvalidFixtureSource;

impl NnsSubnetTopologySource for InvalidFixtureSource {
    fn fetch_subnet_topology_report(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsSubnetTopologyReport, NnsSubnetTopologyHostError> {
        let mut report = fixture_report(2, &request.fetched_at);
        report.subnets[0].node_providers[0].node_count = 0;
        Ok(report)
    }
}

fn refresh_request(root: &std::path::Path, now_unix_secs: u64) -> NnsSubnetTopologyRefreshRequest {
    NnsSubnetTopologyRefreshRequest::new(
        NnsSubnetTopologyCacheRequest::new(root, MAINNET_NETWORK),
        DEFAULT_NNS_SUBNET_TOPOLOGY_SOURCE_ENDPOINT,
        now_unix_secs,
        DEFAULT_NNS_SUBNET_TOPOLOGY_REFRESH_LOCK_STALE_SECONDS,
    )
}

fn fixture_report(registry_version: u64, fetched_at: &str) -> NnsSubnetTopologyReport {
    let mut subnet_principals = principals(&["subnet-a", "subnet-b"]);
    subnet_principals.sort();
    let mut provider_principals = principals(&["provider-a", "provider-b"]);
    provider_principals.sort();
    NnsSubnetTopologyReport {
        schema_version: NNS_SUBNET_TOPOLOGY_REPORT_SCHEMA_VERSION,
        network: MAINNET_NETWORK.to_string(),
        registry_canister_id: MAINNET_REGISTRY_CANISTER_ID.to_string(),
        registry_version,
        fetched_at: fetched_at.to_string(),
        source_endpoint: DEFAULT_NNS_SUBNET_TOPOLOGY_SOURCE_ENDPOINT.to_string(),
        fetched_by: "fixture".to_string(),
        subnet_count: 2,
        node_count: 3,
        subnets: vec![
            NnsSubnetTopologyRow {
                subnet_principal: subnet_principals[0].clone(),
                subnet_kind: SubnetKind::CloudEngine,
                node_count: 2,
                node_providers: vec![
                    NnsSubnetNodeProviderRow {
                        node_provider_principal: provider_principals[0].clone(),
                        node_count: 1,
                    },
                    NnsSubnetNodeProviderRow {
                        node_provider_principal: provider_principals[1].clone(),
                        node_count: 1,
                    },
                ],
            },
            NnsSubnetTopologyRow {
                subnet_principal: subnet_principals[1].clone(),
                subnet_kind: SubnetKind::System,
                node_count: 1,
                node_providers: vec![NnsSubnetNodeProviderRow {
                    node_provider_principal: provider_principals[1].clone(),
                    node_count: 1,
                }],
            },
        ],
    }
}

fn principals(labels: &[&str]) -> Vec<String> {
    labels
        .iter()
        .map(|label| Principal::self_authenticating(label.as_bytes()).to_text())
        .collect()
}
