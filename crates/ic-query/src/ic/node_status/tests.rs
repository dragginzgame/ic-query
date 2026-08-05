use super::*;
use crate::ic::IcDashboardReportProvenance;
#[cfg(feature = "host")]
use crate::{
    HostCacheError,
    ic::{IcHostError, IcNodeStatusSource, IcNodeStatusSourceData, IcSourceRequest},
    progress::IgnoreQueryProgress,
    subnet_catalog::parse_utc_timestamp_secs,
};
#[cfg(feature = "host")]
use std::{cell::Cell, fs, path::PathBuf, time::SystemTime};

#[test]
fn raw_operational_status_classification_is_conservative() {
    for (raw, expected) in [
        ("UP", IcNodeOperationalStatus::Up),
        ("DOWN", IcNodeOperationalStatus::Down),
        ("DISABLED", IcNodeOperationalStatus::Disabled),
        ("DEGRADED", IcNodeOperationalStatus::Degraded),
        ("FUTURE_STATUS", IcNodeOperationalStatus::Unknown),
    ] {
        assert_eq!(IcNodeOperationalStatus::from_raw(raw), expected);
    }
}

#[test]
fn node_count_comparison_covers_every_ordering() {
    for (left, right, expected) in [
        (1, 2, IcNodeCountComparison::Less),
        (2, 2, IcNodeCountComparison::Equal),
        (3, 2, IcNodeCountComparison::Greater),
    ] {
        assert_eq!(IcNodeCountComparison::from_counts(left, right), expected);
    }
}

#[test]
fn all_status_views_share_one_snapshot_and_attention_filter() {
    let snapshot = fixture_snapshot();

    let nodes = ic_node_status_report_from_snapshot(&snapshot, &IcNodeStatusView::attention())
        .expect("node report");
    let subnets = ic_subnet_status_report_from_snapshot(&snapshot, &IcNodeStatusView::attention())
        .expect("Subnet report");
    let providers =
        ic_node_provider_status_report_from_snapshot(&snapshot, &IcNodeStatusView::attention())
            .expect("provider report");

    assert_eq!(nodes.returned_node_count, 2);
    assert_eq!(subnets.returned_subnet_count, 1);
    assert_eq!(providers.returned_provider_count, 2);
    let assigned_provider = providers
        .providers
        .iter()
        .find(|provider| provider.node_provider_id == "aaaaa-aa")
        .expect("assigned provider");
    assert_eq!(assigned_provider.counts.assignment_statuses.assigned.up, 1);
    assert_eq!(
        assigned_provider.unassigned_up_vs_assigned_up,
        IcNodeCountComparison::Less
    );
    assert_eq!(
        assigned_provider.unassigned_non_up_vs_assigned_non_up,
        IcNodeCountComparison::Less
    );
    assert_eq!(
        assigned_provider.counts.assignment_statuses.assigned.down,
        1
    );
    let unassigned_provider = providers
        .providers
        .iter()
        .find(|provider| provider.node_provider_id == "2vxsx-fae")
        .expect("unassigned provider");
    assert_eq!(
        unassigned_provider
            .counts
            .assignment_statuses
            .unassigned
            .degraded,
        1
    );
    assert_eq!(
        unassigned_provider.unassigned_up_vs_assigned_up,
        IcNodeCountComparison::Equal
    );
    assert_eq!(
        unassigned_provider.unassigned_non_up_vs_assigned_non_up,
        IcNodeCountComparison::Greater
    );
    assert_eq!(
        providers.unassigned_up_vs_assigned_up_provider_counts,
        IcNodeCountComparisonCounts {
            less: 1,
            equal: 1,
            greater: 0,
        }
    );
    assert_eq!(
        providers.unassigned_non_up_vs_assigned_non_up_provider_counts,
        IcNodeCountComparisonCounts {
            less: 1,
            equal: 0,
            greater: 1,
        }
    );
    assert_eq!(nodes.observation, subnets.observation);
    assert_eq!(nodes.observation, providers.observation);
}

#[test]
fn status_text_separates_preambles_and_tables() {
    let snapshot = fixture_snapshot();
    let view = IcNodeStatusView::attention().with_all(true);
    let nodes = ic_node_status_report_from_snapshot(&snapshot, &view).expect("node report");
    let subnets = ic_subnet_status_report_from_snapshot(&snapshot, &view).expect("Subnet report");
    let providers =
        ic_node_provider_status_report_from_snapshot(&snapshot, &view).expect("provider report");
    let node_text = ic_node_status_report_text(&nodes);
    let subnet_text = ic_subnet_status_report_text(&subnets);
    let provider_text = ic_node_provider_status_report_text(&providers);

    assert!(
        node_text.contains("unknown=0/0/0\n\nNODE"),
        "node preamble must be visually separate from its table"
    );
    assert!(
        subnet_text.contains("assigned_nodes=2\n\nSUBNET"),
        "Subnet preamble must be visually separate from its table"
    );
    assert!(
        subnet_text.contains("\n\nnon-up node evidence:\nNODE"),
        "later evidence must be a separate visual section"
    );
    assert!(
        provider_text.contains("non_up=1/0/1\n\nNODE PROVIDER"),
        "provider preamble must be visually separate from its table"
    );
    assert!(provider_text.contains("UNASN VS ASN UP/NON-UP"));
    assert!(provider_text.contains("less/less"));
    assert!(provider_text.contains("equal/greater"));
}

#[test]
fn subnet_threshold_reports_down_and_conservative_non_up_distance_separately() {
    let report = ic_subnet_status_report_from_snapshot(
        &fixture_snapshot(),
        &IcNodeStatusView::attention().with_all(true),
    )
    .expect("Subnet report");
    let subnet = report.subnets.first().expect("one assigned Subnet");

    assert_eq!(subnet.statuses.total, 2);
    assert_eq!(subnet.fault_tolerance_node_count, 0);
    assert_eq!(subnet.additional_down_nodes_to_exceed_fault_tolerance, 0);
    assert_eq!(subnet.additional_non_up_nodes_to_exceed_fault_tolerance, 0);
    assert!(subnet.down_fault_tolerance_exceeded);
    assert!(subnet.conservative_non_up_fault_tolerance_exceeded);
}

#[test]
fn target_resolution_supports_unique_prefixes_and_rejects_ambiguity() {
    let snapshot = fixture_snapshot();
    let selected = ic_node_status_report_from_snapshot(
        &snapshot,
        &IcNodeStatusView::attention().with_target("ryj"),
    )
    .expect("unique prefix");
    assert_eq!(selected.returned_node_count, 1);
    assert_eq!(selected.nodes[0].status, "UP");

    let ambiguous = ic_node_status_report_from_snapshot(
        &snapshot,
        &IcNodeStatusView::attention().with_target("r"),
    )
    .expect_err("shared prefix is ambiguous");
    assert!(matches!(
        ambiguous,
        IcNodeStatusProjectionError::AmbiguousTarget { prefix, matches, .. }
            if prefix == "r" && matches.len() == 2
    ));
}

#[test]
fn pure_projections_reject_invalid_raw_relation_evidence() {
    let mut snapshot = fixture_snapshot();
    snapshot.nodes[0].node_type = "UNASSIGNED".to_string();

    let error = ic_subnet_status_report_from_snapshot(
        &snapshot,
        &IcNodeStatusView::attention().with_all(true),
    )
    .expect_err("assigned UNASSIGNED node must be rejected");

    assert!(matches!(
        error,
        IcNodeStatusProjectionError::InvalidSnapshot { reason }
            if reason.contains("assigned subnet evidence")
    ));

    let mut invalid_scope = fixture_snapshot();
    invalid_scope.observation.cloud_engine_nodes_included = true;
    let scope_error = ic_node_status_report_from_snapshot(
        &invalid_scope,
        &IcNodeStatusView::attention().with_all(true),
    )
    .expect_err("unsupported source scope must be rejected");
    assert!(matches!(
        scope_error,
        IcNodeStatusProjectionError::InvalidSnapshot { reason }
            if reason.contains("default mainnet node scope")
    ));
}

#[cfg(feature = "host")]
#[test]
fn non_mainnet_refresh_fails_before_calling_the_source() {
    let source = FixtureSource::default();
    let request = IcNodeStatusRefreshRequest::new(
        temp_dir("ic-node-status-network"),
        "local",
        "https://example.test/api/v3",
        fixture_now(),
        DEFAULT_IC_NODE_STATUS_REFRESH_LOCK_STALE_SECONDS,
    );

    let error = refresh_ic_node_status_snapshot_with_source(&request, &source)
        .expect_err("local network is unsupported");

    assert!(matches!(
        error,
        IcNodeStatusHostError::UnsupportedNetwork { network } if network == "local"
    ));
    assert_eq!(source.calls.get(), 0);
}

#[cfg(feature = "host")]
#[test]
fn stale_reads_reuse_fresh_cache_and_refresh_expired_cache() {
    let root = temp_dir("ic-node-status-stale");
    let source = FixtureSource::default();
    let now = fixture_now();
    let request = refresh_request(&root, now);
    let mut events = Vec::new();
    let mut progress = |event| events.push(event);

    let initial =
        load_or_refresh_stale_ic_node_status_snapshot_with_source(&request, &source, &mut progress)
            .expect("missing cache refreshes");
    assert_eq!(initial.node_count, 1);
    assert_eq!(source.calls.get(), 1);

    let fresh_request = refresh_request(&root, now + DEFAULT_IC_NODE_STATUS_STALE_AFTER_SECONDS);
    load_or_refresh_stale_ic_node_status_snapshot_with_source(
        &fresh_request,
        &source,
        &mut progress,
    )
    .expect("cache at threshold remains fresh");
    assert_eq!(source.calls.get(), 1);

    let stale_request =
        refresh_request(&root, now + DEFAULT_IC_NODE_STATUS_STALE_AFTER_SECONDS + 1);
    let stale = load_or_refresh_stale_ic_node_status_snapshot_with_source(
        &stale_request,
        &source,
        &mut progress,
    )
    .expect("expired cache refreshes");
    assert_eq!(source.calls.get(), 2);
    assert_eq!(
        stale.observation.cache.expect("cache evidence").age_seconds,
        0
    );
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|event| matches!(
        event,
        crate::QueryProgressEvent::CacheRefresh { component, .. }
            if component == "IC node status"
    )));

    fs::remove_dir_all(root).expect("remove fixture cache");
}

#[cfg(feature = "host")]
#[test]
fn malformed_cache_recovers_while_strict_load_remains_typed() {
    let root = temp_dir("ic-node-status-invalid");
    let source = FixtureSource::default();
    let now = fixture_now();
    let request = refresh_request(&root, now);
    let cache_path = ic_node_status_cache_path(&root, "ic");
    fs::create_dir_all(cache_path.parent().expect("cache parent")).expect("create cache parent");
    fs::write(&cache_path, "not-json").expect("write malformed cache");

    let strict = load_cached_ic_node_status_snapshot(&request.cache, now)
        .expect_err("strict load rejects malformed cache");
    assert!(matches!(
        strict,
        IcNodeStatusHostError::Cache(HostCacheError::ParseCache { .. })
    ));

    let mut progress = IgnoreQueryProgress;
    let recovered = load_or_refresh_missing_ic_node_status_snapshot_with_source(
        &request,
        &source,
        &mut progress,
    )
    .expect("read-through policy repairs malformed cache");
    assert_eq!(recovered.node_count, 1);
    assert_eq!(source.calls.get(), 1);

    fs::remove_dir_all(root).expect("remove fixture cache");
}

#[cfg(feature = "host")]
#[test]
fn noncanonical_cache_order_is_invalid_and_read_through_repairs_it() {
    let root = temp_dir("ic-node-status-cache-order");
    let now = fixture_now();
    let request = refresh_request(&root, now);
    let source = RowsSource {
        nodes: fixture_snapshot().nodes,
    };
    refresh_ic_node_status_snapshot_with_source(&request, &source).expect("create cache");
    let cache_path = ic_node_status_cache_path(&root, "ic");
    let mut cache: serde_json::Value =
        serde_json::from_slice(&fs::read(&cache_path).expect("read fixture cache"))
            .expect("parse fixture cache");
    let nodes = cache["nodes"].as_array_mut().expect("cached node rows");
    nodes.swap(0, 1);
    fs::write(
        &cache_path,
        serde_json::to_vec_pretty(&cache).expect("serialize reordered cache"),
    )
    .expect("write reordered cache");

    let strict = load_cached_ic_node_status_snapshot(&request.cache, now)
        .expect_err("strict load rejects noncanonical cache ordering");
    assert!(matches!(
        strict,
        IcNodeStatusHostError::InvalidCache { reason, .. }
            if reason.contains("strict canonical node-id order")
    ));

    let repair_source = FixtureSource::default();
    let mut progress = IgnoreQueryProgress;
    let repaired = load_or_refresh_missing_ic_node_status_snapshot_with_source(
        &request,
        &repair_source,
        &mut progress,
    )
    .expect("read-through repairs noncanonical cache ordering");
    assert_eq!(repaired.node_count, 1);
    assert_eq!(repair_source.calls.get(), 1);

    fs::remove_dir_all(root).expect("remove fixture cache");
}

#[cfg(feature = "host")]
#[test]
fn forced_refresh_reports_atomic_replacement() {
    let root = temp_dir("ic-node-status-force");
    let source = FixtureSource::default();
    let request = refresh_request(&root, fixture_now());

    let created = refresh_ic_node_status_snapshot_with_source(&request, &source)
        .expect("create complete cache");
    let replaced = refresh_ic_node_status_snapshot_with_source(&request, &source)
        .expect("replace complete cache");

    assert!(!created.replaced_existing_cache);
    assert!(replaced.replaced_existing_cache);
    assert_eq!(source.calls.get(), 2);
    assert_eq!(
        load_cached_ic_node_status_snapshot(&request.cache, request.now_unix_secs)
            .expect("strictly load replacement")
            .node_count,
        1
    );

    fs::remove_dir_all(root).expect("remove fixture cache");
}

#[cfg(feature = "host")]
#[test]
fn custom_source_rows_are_sorted_and_preserve_unknown_status() {
    let source = RowsSource {
        nodes: vec![
            fixture_node(
                "rrkah-fqaaa-aaaaa-aaaaq-cai",
                "FUTURE_STATUS",
                "UNASSIGNED",
                None,
                "2vxsx-fae",
            ),
            fixture_node(
                "aaaaa-aa",
                "UP",
                "REPLICA",
                Some("ryjl3-tyaaa-aaaaa-aaaba-cai"),
                "aaaaa-aa",
            ),
        ],
    };
    let request = IcNodeStatusSnapshotRequest::new("https://example.test/api/v3", fixture_now());

    let snapshot = crate::ic::build_ic_node_status_snapshot_with_source(&request, &source)
        .expect("valid custom snapshot");

    assert_eq!(snapshot.nodes[0].node_id, "aaaaa-aa");
    assert_eq!(snapshot.counts.statuses.unknown, 1);
    assert_eq!(snapshot.counts.statuses.up, 1);
}

#[cfg(feature = "host")]
#[test]
fn custom_source_rejects_duplicate_nodes_and_inconsistent_provider_names() {
    let request = IcNodeStatusSnapshotRequest::new("https://example.test/api/v3", fixture_now());
    let duplicate = fixture_node(
        "aaaaa-aa",
        "UP",
        "REPLICA",
        Some("ryjl3-tyaaa-aaaaa-aaaba-cai"),
        "2vxsx-fae",
    );
    let duplicate_error = crate::ic::build_ic_node_status_snapshot_with_source(
        &request,
        &RowsSource {
            nodes: vec![duplicate.clone(), duplicate],
        },
    )
    .expect_err("duplicate node id");
    assert!(matches!(
        duplicate_error,
        IcHostError::InvalidSourceData { .. }
    ));

    let mut conflicting = fixture_node(
        "rrkah-fqaaa-aaaaa-aaaaq-cai",
        "UP",
        "UNASSIGNED",
        None,
        "2vxsx-fae",
    );
    conflicting.node_provider_name = "Conflicting provider name".to_string();
    let provider_error = crate::ic::build_ic_node_status_snapshot_with_source(
        &request,
        &RowsSource {
            nodes: vec![
                fixture_node(
                    "aaaaa-aa",
                    "UP",
                    "REPLICA",
                    Some("ryjl3-tyaaa-aaaaa-aaaba-cai"),
                    "2vxsx-fae",
                ),
                conflicting,
            ],
        },
    )
    .expect_err("provider name conflict");
    assert!(matches!(
        provider_error,
        IcHostError::InvalidSourceData { .. }
    ));
}

#[cfg(feature = "host")]
#[test]
fn custom_source_rejects_invalid_principals_assignment_and_scope_evidence() {
    let request = IcNodeStatusSnapshotRequest::new("https://example.test/api/v3", fixture_now());
    let mut invalid_principal = fixture_node(
        "aaaaa-aa",
        "UP",
        "REPLICA",
        Some("ryjl3-tyaaa-aaaaa-aaaba-cai"),
        "2vxsx-fae",
    );
    invalid_principal.node_id = "not-a-principal".to_string();
    let contradictory = fixture_node(
        "aaaaa-aa",
        "UP",
        "UNASSIGNED",
        Some("ryjl3-tyaaa-aaaaa-aaaba-cai"),
        "2vxsx-fae",
    );
    let mut cloud_engine = fixture_node(
        "aaaaa-aa",
        "UP",
        "REPLICA",
        Some("ryjl3-tyaaa-aaaaa-aaaba-cai"),
        "2vxsx-fae",
    );
    cloud_engine.cloud_engine_subnet_id = Some("rrkah-fqaaa-aaaaa-aaaaq-cai".to_string());

    for node in [invalid_principal, contradictory, cloud_engine] {
        let error = crate::ic::build_ic_node_status_snapshot_with_source(
            &request,
            &RowsSource { nodes: vec![node] },
        )
        .expect_err("invalid source row");
        assert!(matches!(error, IcHostError::InvalidSourceData { .. }));
    }
}

#[cfg(feature = "host")]
#[test]
fn custom_source_rejects_rows_above_the_fixed_ceiling() {
    let row = fixture_node(
        "aaaaa-aa",
        "UP",
        "REPLICA",
        Some("ryjl3-tyaaa-aaaaa-aaaba-cai"),
        "2vxsx-fae",
    );
    let row_count = usize::try_from(MAX_IC_NODE_STATUS_ROWS).expect("row ceiling") + 1;
    let request = IcNodeStatusSnapshotRequest::new("https://example.test/api/v3", fixture_now());

    let error = crate::ic::build_ic_node_status_snapshot_with_source(
        &request,
        &RowsSource {
            nodes: vec![row; row_count],
        },
    )
    .expect_err("oversized source collection");

    assert!(matches!(error, IcHostError::InvalidSourceData { .. }));
}

#[cfg(feature = "host")]
#[test]
fn custom_source_rejects_an_empty_mainnet_snapshot() {
    let request = IcNodeStatusSnapshotRequest::new("https://example.test/api/v3", fixture_now());

    let error = crate::ic::build_ic_node_status_snapshot_with_source(
        &request,
        &RowsSource { nodes: Vec::new() },
    )
    .expect_err("empty mainnet snapshot");

    assert!(matches!(
        error,
        IcHostError::InvalidSourceData { reason }
            if reason.contains("at least one row")
    ));
}

fn fixture_snapshot() -> IcNodeStatusSnapshot {
    let subnet_id = "ryjl3-tyaaa-aaaaa-aaaba-cai";
    let mut nodes = vec![
        fixture_node("aaaaa-aa", "DOWN", "REPLICA", Some(subnet_id), "aaaaa-aa"),
        fixture_node(
            "ryjl3-tyaaa-aaaaa-aaaba-cai",
            "UP",
            "REPLICA",
            Some(subnet_id),
            "aaaaa-aa",
        ),
        fixture_node(
            "rrkah-fqaaa-aaaaa-aaaaq-cai",
            "DEGRADED",
            "UNASSIGNED",
            None,
            "2vxsx-fae",
        ),
    ];
    nodes.sort_unstable_by(|left, right| left.node_id.cmp(&right.node_id));
    let counts = node_status_group_counts(nodes.iter());
    IcNodeStatusSnapshot {
        observation: IcNodeStatusObservation {
            source: IcDashboardReportProvenance {
                schema_version: IC_NODE_STATUS_SCHEMA_VERSION,
                network: "ic".to_string(),
                authority: "official_ic_dashboard_api".to_string(),
                source_endpoint: "https://example.test/api/v3".to_string(),
                fetched_at: "2026-08-04T12:00:00Z".to_string(),
                fetched_by: "fixture".to_string(),
                certified: false,
                point_in_time_guaranteed: false,
            },
            scope: IcNodeStatusScope::DashboardMainnetDefault,
            cloud_engine_nodes_included: false,
            cache: None,
        },
        node_count: 3,
        counts,
        nodes,
    }
}

fn fixture_node(
    node_id: &str,
    status: &str,
    node_type: &str,
    subnet_id: Option<&str>,
    node_provider_id: &str,
) -> IcNodeStatusRow {
    IcNodeStatusRow {
        node_id: node_id.to_string(),
        node_operator_id: "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string(),
        node_provider_id: node_provider_id.to_string(),
        node_provider_name: format!("Provider {node_provider_id}"),
        node_type: node_type.to_string(),
        node_reward_type: "Type3".to_string(),
        status: status.to_string(),
        alert_name: (status != "UP").then(|| "fixture alert".to_string()),
        subnet_id: subnet_id.map(str::to_string),
        cloud_engine_subnet_id: None,
        data_center_id: "dc1".to_string(),
        data_center_name: "Data center".to_string(),
        owner: "Owner".to_string(),
        region: "Europe,FR".to_string(),
        guestos_version: Some("version".to_string()),
        guestos_tee_active: Some(false),
        ip_address: None,
        ipv4_connectivity_status: Some(true),
        node_hardware_generation: None,
    }
}

#[cfg(feature = "host")]
#[derive(Default)]
struct FixtureSource {
    calls: Cell<usize>,
}

#[cfg(feature = "host")]
struct RowsSource {
    nodes: Vec<IcNodeStatusRow>,
}

#[cfg(feature = "host")]
impl IcNodeStatusSource for RowsSource {
    fn fetch_node_status_snapshot(
        &self,
        request: &IcSourceRequest,
    ) -> Result<IcNodeStatusSourceData, IcHostError> {
        Ok(IcNodeStatusSourceData {
            source: request.clone(),
            scope: IcNodeStatusScope::DashboardMainnetDefault,
            cloud_engine_nodes_included: false,
            nodes: self.nodes.clone(),
        })
    }
}

#[cfg(feature = "host")]
impl IcNodeStatusSource for FixtureSource {
    fn fetch_node_status_snapshot(
        &self,
        request: &IcSourceRequest,
    ) -> Result<IcNodeStatusSourceData, IcHostError> {
        self.calls.set(self.calls.get() + 1);
        Ok(IcNodeStatusSourceData {
            source: request.clone(),
            scope: IcNodeStatusScope::DashboardMainnetDefault,
            cloud_engine_nodes_included: false,
            nodes: vec![fixture_node(
                "aaaaa-aa",
                "DOWN",
                "REPLICA",
                Some("ryjl3-tyaaa-aaaaa-aaaba-cai"),
                "2vxsx-fae",
            )],
        })
    }
}

#[cfg(feature = "host")]
fn refresh_request(root: &std::path::Path, now_unix_secs: u64) -> IcNodeStatusRefreshRequest {
    IcNodeStatusRefreshRequest::new(
        root,
        "ic",
        "https://example.test/api/v3",
        now_unix_secs,
        DEFAULT_IC_NODE_STATUS_REFRESH_LOCK_STALE_SECONDS,
    )
}

#[cfg(feature = "host")]
fn fixture_now() -> u64 {
    parse_utc_timestamp_secs("2026-08-04T12:00:00Z").expect("fixture timestamp")
}

#[cfg(feature = "host")]
fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{nonce}"))
}
