#[cfg(feature = "host")]
use ic_query::nns::data_center::{
    DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS, DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
    NnsDataCenterRefreshReport, NnsDataCenterRefreshRequest, build_nns_data_center_info_report,
    build_nns_data_center_list_report, nns_data_center_cache_path,
    nns_data_center_refresh_lock_path, nns_data_center_refresh_report_text,
    refresh_nns_data_center_report,
};
use ic_query::nns::data_center::{
    NnsDataCenterCacheRequest, NnsDataCenterInfoReport, NnsDataCenterInfoRequest,
    NnsDataCenterListReport, NnsDataCenterListRequest, NnsDataCenterRow,
    nns_data_center_info_report_text, nns_data_center_list_report_text,
    nns_data_center_list_report_verbose_text,
};
#[cfg(feature = "host")]
use ic_query::nns::node::{
    DEFAULT_NNS_NODE_SOURCE_ENDPOINT, DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS,
    NnsNodeRefreshReport, NnsNodeRefreshRequest, build_nns_node_info_report,
    build_nns_node_list_report, nns_node_cache_path, nns_node_refresh_lock_path,
    nns_node_refresh_report_text, refresh_nns_node_report,
};
use ic_query::nns::node::{
    NNS_NODE_SUBNET_KIND_APPLICATION, NnsNodeCacheRequest, NnsNodeInfoReport, NnsNodeInfoRequest,
    NnsNodeListFilters, NnsNodeListReport, NnsNodeListRequest, NnsNodeRow,
    nns_node_info_report_text, nns_node_list_report_text, nns_node_list_report_verbose_text,
};
#[cfg(feature = "host")]
use ic_query::nns::node_operator::{
    DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT, DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS,
    NnsNodeOperatorRefreshReport, NnsNodeOperatorRefreshRequest,
    build_nns_node_operator_info_report, build_nns_node_operator_list_report,
    nns_node_operator_cache_path, nns_node_operator_refresh_lock_path,
    nns_node_operator_refresh_report_text, refresh_nns_node_operator_report,
};
use ic_query::nns::node_operator::{
    NnsNodeOperatorCacheRequest, NnsNodeOperatorInfoReport, NnsNodeOperatorInfoRequest,
    NnsNodeOperatorListReport, NnsNodeOperatorListRequest, NnsNodeOperatorRow,
    nns_node_operator_info_report_text, nns_node_operator_list_report_text,
    nns_node_operator_list_report_verbose_text,
};
#[cfg(feature = "host")]
use ic_query::nns::node_provider::{
    DEFAULT_NNS_SOURCE_ENDPOINT, DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS,
    NnsNodeProviderRefreshReport, NnsNodeProviderRefreshRequest,
    build_nns_node_provider_info_report, build_nns_node_provider_list_report,
    nns_node_provider_cache_path, nns_node_provider_refresh_lock_path,
    nns_node_provider_refresh_report_text, refresh_nns_node_provider_report,
};
use ic_query::nns::node_provider::{
    NnsNodeProviderCacheRequest, NnsNodeProviderInfoReport, NnsNodeProviderInfoRequest,
    NnsNodeProviderListReport, NnsNodeProviderListRequest, NnsNodeProviderRow,
    nns_node_provider_info_report_text, nns_node_provider_list_report_text,
    nns_node_provider_list_report_verbose_text,
};
use ic_query::nns::proposals::{
    NnsProposalBallotRow, NnsProposalListReport, NnsProposalListRequest, NnsProposalListSort,
    NnsProposalReport, NnsProposalRequest, NnsProposalRewardStatusFilter, NnsProposalRow,
    NnsProposalSortDirection, NnsProposalStatusFilter, NnsProposalTally, NnsProposalTopicFilter,
    nns_proposal_list_report_text, nns_proposal_report_text,
};
use ic_query::nns::registry::{
    NnsRegistryVersionReport, NnsRegistryVersionRequest, nns_registry_version_report_text,
};
use ic_query::nns::topology::report::{
    NnsTopologyCapacityReport, NnsTopologyCapacityRow, NnsTopologyCoverageReport,
    NnsTopologyGapRow, NnsTopologyGapsReport, NnsTopologyHealthCheckRow, NnsTopologyHealthReport,
    NnsTopologyProviderRow, NnsTopologyProvidersReport, NnsTopologyRefreshReport,
    NnsTopologyRefreshRequest, NnsTopologyRefreshRow, NnsTopologyRegionRow,
    NnsTopologyRegionsReport, NnsTopologyRegistryVersionRow, NnsTopologySummaryReport,
    NnsTopologySummaryRequest, NnsTopologyVersionsReport, nns_topology_capacity_report_text,
    nns_topology_coverage_report_text, nns_topology_gaps_report_text,
    nns_topology_health_report_text, nns_topology_providers_report_text,
    nns_topology_refresh_report_text, nns_topology_regions_report_text,
    nns_topology_summary_report_text, nns_topology_versions_report_text,
};
#[cfg(feature = "host")]
use serde::Serialize;
#[cfg(feature = "host")]
use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn public_nns_registry_api_is_constructible_and_renderable() {
    let request = NnsRegistryVersionRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
    };

    assert_eq!(request.network, "ic");

    let report = NnsRegistryVersionReport {
        schema_version: 1,
        network: request.network,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
    };

    let text = nns_registry_version_report_text(&report);

    assert!(text.contains("network: ic"));
    assert!(text.contains("registry_version: 42"));
}

#[test]
fn public_nns_node_api_is_constructible_and_renderable() {
    let cache = NnsNodeCacheRequest {
        icp_root: ".".into(),
        network: "ic".to_string(),
    };
    let filters = NnsNodeListFilters {
        subnet: Some("tdb26-jop6g".to_string()),
        subnet_kind: Some(NNS_NODE_SUBNET_KIND_APPLICATION.to_string()),
        data_center: Some("zh1".to_string()),
        node_provider: None,
        node_operator: None,
    };
    let list_request = NnsNodeListRequest {
        cache: cache.clone(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        filters,
    };

    assert_eq!(
        list_request.filters.subnet_kind.as_deref(),
        Some(NNS_NODE_SUBNET_KIND_APPLICATION)
    );

    let node = sample_nns_node_row();
    let list_report = NnsNodeListReport {
        schema_version: 1,
        network: list_request.cache.network,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: list_request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        node_count: 1,
        nodes: vec![node.clone()],
    };

    let list_text = nns_node_list_report_text(&list_report);
    let verbose_text = nns_node_list_report_verbose_text(&list_report);

    assert!(list_text.contains("nodes: ic count 1"));
    assert!(list_text.contains(NNS_NODE_SUBNET_KIND_APPLICATION));
    assert!(verbose_text.contains("source_endpoint: https://icp-api.io"));
    assert!(verbose_text.contains("tdb26-jop6g-7sc54-foywl"));

    let info_request = NnsNodeInfoRequest {
        cache,
        source_endpoint: "https://icp-api.io".to_string(),
        input: node.node_principal.clone(),
        now_unix_secs: 1_700_000_000,
    };
    let info_report = NnsNodeInfoReport {
        schema_version: 1,
        input: info_request.input,
        resolved_from: "node_principal".to_string(),
        network: info_request.cache.network,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: info_request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        node_principal: node.node_principal,
        node_operator_principal: node.node_operator_principal,
        node_provider_principal: node.node_provider_principal,
        subnet_principal: node.subnet_principal,
        subnet_kind: node.subnet_kind,
        data_center_id: node.data_center_id,
    };

    let info_text = nns_node_info_report_text(&info_report);

    assert!(info_text.contains("resolved_from: node_principal"));
    assert!(info_text.contains("data_center_id: zh1"));
}

#[test]
fn public_nns_data_center_api_is_constructible_and_renderable() {
    let cache = NnsDataCenterCacheRequest {
        icp_root: ".".into(),
        network: "ic".to_string(),
    };
    let request = NnsDataCenterListRequest {
        cache: cache.clone(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
    };
    let data_center = sample_nns_data_center_row();
    let list = NnsDataCenterListReport {
        schema_version: 1,
        network: request.cache.network,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        data_center_count: 1,
        data_centers: vec![data_center.clone()],
    };

    let text = nns_data_center_list_report_text(&list);
    let verbose_text = nns_data_center_list_report_verbose_text(&list);

    assert!(text.contains("data_centers: ic count 1"));
    assert!(text.contains("Zurich"));
    assert!(verbose_text.contains("REGISTRY_VERSION"));

    let info_request = NnsDataCenterInfoRequest {
        cache,
        source_endpoint: "https://icp-api.io".to_string(),
        input: data_center.data_center_id.clone(),
        now_unix_secs: 1_700_000_000,
    };
    let info = NnsDataCenterInfoReport {
        schema_version: 1,
        input: info_request.input,
        resolved_from: "data_center_id".to_string(),
        network: info_request.cache.network,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: info_request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        data_center_id: data_center.data_center_id,
        region: data_center.region,
        owner: data_center.owner,
        latitude: data_center.latitude,
        longitude: data_center.longitude,
        node_operator_count: data_center.node_operator_count,
        node_provider_count: data_center.node_provider_count,
        node_count: data_center.node_count,
    };

    let info_text = nns_data_center_info_report_text(&info);

    assert!(info_text.contains("resolved_from: data_center_id"));
    assert!(info_text.contains("node_count: 12"));
}

#[test]
fn public_nns_node_provider_api_is_constructible_and_renderable() {
    let cache = NnsNodeProviderCacheRequest {
        icp_root: ".".into(),
        network: "ic".to_string(),
    };
    let request = NnsNodeProviderListRequest {
        cache: cache.clone(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
    };
    let provider = sample_nns_node_provider_row();
    let list = NnsNodeProviderListReport {
        schema_version: 1,
        network: request.cache.network,
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        node_provider_count: 1,
        node_providers: vec![provider.clone()],
    };

    let text = nns_node_provider_list_report_text(&list);
    let verbose_text = nns_node_provider_list_report_verbose_text(&list);

    assert!(text.contains("node_providers: ic count 1"));
    assert!(text.contains("12"));
    assert!(verbose_text.contains("deadbeef"));

    let info_request = NnsNodeProviderInfoRequest {
        cache,
        source_endpoint: "https://icp-api.io".to_string(),
        input: provider.node_provider_principal.clone(),
        now_unix_secs: 1_700_000_000,
    };
    let info = NnsNodeProviderInfoReport {
        schema_version: 1,
        input: info_request.input,
        resolved_from: "node_provider_principal".to_string(),
        network: info_request.cache.network,
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: info_request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        node_provider_principal: provider.node_provider_principal,
        name: provider.name,
        node_count: provider.node_count,
        reward_account_hex: provider.reward_account_hex,
    };

    let info_text = nns_node_provider_info_report_text(&info);

    assert!(info_text.contains("resolved_from: node_provider_principal"));
    assert!(info_text.contains("reward_account_hex: deadbeef"));
}

#[test]
fn public_nns_node_operator_api_is_constructible_and_renderable() {
    let cache = NnsNodeOperatorCacheRequest {
        icp_root: ".".into(),
        network: "ic".to_string(),
    };
    let request = NnsNodeOperatorListRequest {
        cache: cache.clone(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
    };
    let operator = sample_nns_node_operator_row();
    let list = NnsNodeOperatorListReport {
        schema_version: 1,
        network: request.cache.network,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        node_operator_count: 1,
        node_operators: vec![operator.clone()],
    };

    let text = nns_node_operator_list_report_text(&list);
    let verbose_text = nns_node_operator_list_report_verbose_text(&list);

    assert!(text.contains("node_operators: ic count 1"));
    assert!(text.contains("zh1"));
    assert!(verbose_text.contains("tdb26-jop6g-7sc54-foywl"));

    let info_request = NnsNodeOperatorInfoRequest {
        cache,
        source_endpoint: "https://icp-api.io".to_string(),
        input: operator.node_operator_principal.clone(),
        now_unix_secs: 1_700_000_000,
    };
    let info = NnsNodeOperatorInfoReport {
        schema_version: 1,
        input: info_request.input,
        resolved_from: "node_operator_principal".to_string(),
        network: info_request.cache.network,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: info_request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        node_operator_principal: operator.node_operator_principal,
        node_provider_principal: operator.node_provider_principal,
        node_allowance: operator.node_allowance,
        data_center_id: operator.data_center_id,
        node_count: operator.node_count,
    };

    let info_text = nns_node_operator_info_report_text(&info);

    assert!(info_text.contains("resolved_from: node_operator_principal"));
    assert!(info_text.contains("node_allowance: 28"));
}

#[cfg(feature = "host")]
#[test]
fn public_nns_inventory_host_api_reads_cached_reports_without_cli() {
    let root = temp_root("nns-inventory-host-public-api");
    write_nns_inventory_fixture_caches(&root);

    assert_public_nns_node_host_api(&root);
    assert_public_nns_data_center_host_api(&root);
    assert_public_nns_node_provider_host_api(&root);
    assert_public_nns_node_operator_host_api(&root);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn public_nns_proposal_api_is_constructible_and_renderable() {
    let request = NnsProposalListRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        limit: 25,
        before_proposal_id: Some(132_500),
        status: NnsProposalStatusFilter::Executed,
        reward_status: NnsProposalRewardStatusFilter::Settled,
        topic: NnsProposalTopicFilter::Governance,
        proposer_neuron_id: Some(12_345),
        query: Some("subnet".to_string()),
        sort: NnsProposalListSort::TallyTime,
        sort_direction: NnsProposalSortDirection::Desc,
        verbose: true,
    };

    assert_eq!(request.status.as_str(), "executed");
    assert_eq!(request.reward_status.as_str(), "settled");
    assert_eq!(request.topic.as_str(), "governance");
    assert_eq!(request.sort.direction_label(request.sort_direction), "desc");

    let proposal = sample_nns_proposal_row();
    let list_report = NnsProposalListReport {
        schema_version: 3,
        network: request.network,
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        data_source: "cache".to_string(),
        cache_path: Some(".icq/nns/ic/governance/proposals/full.json".to_string()),
        cache_complete: Some(true),
        requested_limit: request.limit,
        before_proposal_id: request.before_proposal_id,
        status_filter: request.status.as_str().to_string(),
        reward_status_filter: request.reward_status.as_str().to_string(),
        topic_filter: request.topic.as_str().to_string(),
        proposer_filter: request.proposer_neuron_id,
        query_filter: request.query,
        sort: request.sort.as_str().to_string(),
        sort_direction: request
            .sort
            .direction_label(request.sort_direction)
            .to_string(),
        result_scope: "complete-cache".to_string(),
        verbose: request.verbose,
        proposal_count: 1,
        proposals: vec![proposal.clone()],
    };

    let list_text = nns_proposal_list_report_text(&list_report);

    assert!(list_text.contains("proposal_count: 1"));
    assert!(list_text.contains("topic_filter: governance"));
    assert!(list_text.contains("proposal_details:"));
    assert!(list_text.contains("title: Upgrade subnet"));

    let detail_request = NnsProposalRequest {
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        proposal_id: 132_411,
        show_ballots: true,
        verbose: false,
    };
    let detail_report = NnsProposalReport {
        schema_version: 1,
        network: detail_request.network,
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: detail_request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        data_source: "live".to_string(),
        cache_path: None,
        cache_complete: None,
        proposal_id: detail_request.proposal_id,
        show_ballots: detail_request.show_ballots,
        verbose: detail_request.verbose,
        proposal,
    };

    let detail_text = nns_proposal_report_text(&detail_report);

    assert!(detail_text.contains("proposal_id: 132411"));
    assert!(detail_text.contains("show_ballots: yes"));
    assert!(detail_text.contains("reject_cost: 1.00"));
    assert!(detail_text.contains("ballots:"));
    assert!(detail_text.contains("yes"));
}

#[test]
fn public_nns_topology_summary_and_versions_api_is_constructible_and_renderable() {
    let request = NnsTopologySummaryRequest {
        icp_root: ".".into(),
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
    };
    assert_eq!(request.network, "ic");

    let registry_version = sample_topology_registry_version_row();
    let summary = NnsTopologySummaryReport {
        schema_version: 3,
        network: request.network,
        source_endpoint: request.source_endpoint,
        subnet_count: 2,
        application_subnet_count: 1,
        cloud_engine_subnet_count: 0,
        system_subnet_count: 1,
        unknown_subnet_count: 0,
        routing_range_count: 4,
        node_count: 3,
        application_node_count: 2,
        cloud_engine_node_count: 0,
        system_node_count: 1,
        unknown_node_count: 0,
        node_provider_count: 1,
        node_operator_count: 1,
        data_center_count: 1,
        nodes_with_known_node_provider_count: 3,
        nodes_with_unknown_node_provider_count: 0,
        nodes_with_known_node_operator_count: 3,
        nodes_with_unknown_node_operator_count: 0,
        nodes_with_known_data_center_count: 3,
        nodes_with_unknown_data_center_count: 0,
        node_operators_with_known_node_provider_count: 1,
        node_operators_with_unknown_node_provider_count: 0,
        node_operators_with_known_data_center_count: 1,
        node_operators_with_unknown_data_center_count: 0,
        subnet_catalog_stale: false,
        subnet_catalog_stale_reason: "fresh".to_string(),
        registry_versions: vec![registry_version.clone()],
    };
    let summary_text = nns_topology_summary_report_text(&summary);
    assert!(summary_text.contains("topology: ic subnets 2 nodes 3"));
    assert!(summary_text.contains("subnet_catalog"));

    let versions = NnsTopologyVersionsReport {
        schema_version: 1,
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        source_count: 1,
        registry_versions: vec![registry_version],
    };
    assert!(nns_topology_versions_report_text(&versions).contains("subnet_catalog"));
}

#[test]
fn public_nns_topology_coverage_and_health_api_is_constructible_and_renderable() {
    let health = NnsTopologyHealthReport {
        schema_version: 1,
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        status: "ok".to_string(),
        registry_source_count: 1,
        registry_version_min: Some(42),
        registry_version_max: Some(42),
        registry_versions_aligned: true,
        stale_source_count: 0,
        subnet_catalog_stale: false,
        subnet_catalog_stale_reason: "fresh".to_string(),
        known_join_count: 11,
        unknown_join_count: 0,
        join_coverage: "100.0%".to_string(),
        checks: vec![NnsTopologyHealthCheckRow {
            check: "registry_versions".to_string(),
            status: "ok".to_string(),
            detail: "1 source at registry version 42".to_string(),
        }],
    };
    assert!(nns_topology_health_report_text(&health).contains("registry_versions"));

    let coverage = NnsTopologyCoverageReport {
        schema_version: 1,
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        node_count: 3,
        node_provider_count: 1,
        node_operator_count: 1,
        data_center_count: 1,
        nodes_with_known_node_provider_count: 3,
        nodes_with_unknown_node_provider_count: 0,
        nodes_with_known_node_operator_count: 3,
        nodes_with_unknown_node_operator_count: 0,
        nodes_with_known_data_center_count: 3,
        nodes_with_unknown_data_center_count: 0,
        node_operators_with_known_node_provider_count: 1,
        node_operators_with_unknown_node_provider_count: 0,
        node_operators_with_known_data_center_count: 1,
        node_operators_with_unknown_data_center_count: 0,
    };
    assert!(nns_topology_coverage_report_text(&coverage).contains("nodes"));
}

#[test]
fn public_nns_topology_gaps_and_capacity_api_is_constructible_and_renderable() {
    let gaps = NnsTopologyGapsReport {
        schema_version: 1,
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        status: "attention".to_string(),
        gap_count: 1,
        gaps: vec![NnsTopologyGapRow {
            subject_kind: "node".to_string(),
            subject: "zh3jp-xqaaa-aaaar-qaada-cai".to_string(),
            missing_relation: "node_operator".to_string(),
            referenced_id: "qoctq-giaaa-aaaar-qaada-cai".to_string(),
        }],
    };
    assert!(nns_topology_gaps_report_text(&gaps).contains("node_operator"));

    let capacity = NnsTopologyCapacityReport {
        schema_version: 1,
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        status: "attention".to_string(),
        node_operator_count: 1,
        total_node_allowance: 2,
        assigned_node_count: 3,
        unknown_node_count_operator_count: 0,
        available_node_slots: 0,
        over_assigned_operator_count: 1,
        over_assigned_node_count: 1,
        capacity: vec![NnsTopologyCapacityRow {
            node_operator_principal: "qoctq-giaaa-aaaar-qaada-cai".to_string(),
            node_provider_principal: "w6gnz-6qaaa-aaaar-qaada-cai".to_string(),
            data_center_id: "zh1".to_string(),
            node_allowance: 2,
            assigned_node_count: Some(3),
            available_node_slots: Some(0),
            over_assigned_node_count: Some(1),
            utilization: "150.0%".to_string(),
            status: "over".to_string(),
        }],
    };
    assert!(nns_topology_capacity_report_text(&capacity).contains("over"));
}

#[test]
fn public_nns_topology_region_provider_and_refresh_api_is_constructible_and_renderable() {
    let regions = NnsTopologyRegionsReport {
        schema_version: 1,
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        region_count: 1,
        data_center_count: 1,
        node_operator_count: 1,
        node_provider_count: 1,
        node_count: 3,
        regions: vec![NnsTopologyRegionRow {
            region: "Zurich".to_string(),
            data_center_count: 1,
            node_operator_count: 1,
            node_provider_count: 1,
            node_count: 3,
        }],
    };
    assert!(nns_topology_regions_report_text(&regions).contains("Zurich"));

    let providers = NnsTopologyProvidersReport {
        schema_version: 1,
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        registered_node_provider_count: 1,
        referenced_node_provider_count: 1,
        provider_with_nodes_count: 1,
        provider_with_node_operators_count: 1,
        total_node_count: 3,
        total_node_operator_count: 1,
        total_node_allowance: 2,
        over_assigned_provider_count: 1,
        unknown_provider_count: 0,
        providers: vec![NnsTopologyProviderRow {
            node_provider_principal: "w6gnz-6qaaa-aaaar-qaada-cai".to_string(),
            registered: true,
            name: Some("Example Provider".to_string()),
            governance_node_count: Some(3),
            topology_node_count: 3,
            node_operator_count: 1,
            data_center_count: 1,
            region_count: 1,
            total_node_allowance: 2,
            assigned_node_count: 3,
            available_node_slots: 0,
            over_assigned_node_count: 1,
            status: "over".to_string(),
        }],
    };
    assert!(nns_topology_providers_report_text(&providers).contains("over"));

    let refresh_request = NnsTopologyRefreshRequest {
        icp_root: ".".into(),
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        now_unix_secs: 1_700_000_000,
        lock_stale_after_seconds: 1_800,
        dry_run: true,
    };
    let refresh = NnsTopologyRefreshReport {
        schema_version: 1,
        network: refresh_request.network,
        source_endpoint: refresh_request.source_endpoint,
        dry_run: refresh_request.dry_run,
        component_count: 1,
        wrote_cache_count: 0,
        replaced_existing_cache_count: 0,
        components: vec![NnsTopologyRefreshRow {
            source: "subnet_catalog".to_string(),
            cache_path: ".icq/subnet-catalog/ic/catalog.json".to_string(),
            refresh_lock_path: ".icq/subnet-catalog/ic/refresh.lock".to_string(),
            registry_version: 42,
            fetched_at: "2023-11-14T22:13:20Z".to_string(),
            source_endpoint: "https://icp-api.io".to_string(),
            fetched_by: "ic-query".to_string(),
            dry_run: true,
            wrote_cache: false,
            replaced_existing_cache: false,
            item_count: 2,
        }],
    };
    assert!(nns_topology_refresh_report_text(&refresh).contains("topology_refresh: ic"));
}

#[cfg(feature = "host")]
type RefreshFn<Request, Report, Error> = fn(&Request) -> Result<Report, Error>;

#[cfg(feature = "host")]
fn write_nns_inventory_fixture_caches(root: &Path) {
    write_json_cache(
        &nns_node_cache_path(root, "ic"),
        &sample_nns_node_list_report(),
    );
    write_json_cache(
        &nns_data_center_cache_path(root, "ic"),
        &sample_nns_data_center_list_report(),
    );
    write_json_cache(
        &nns_node_provider_cache_path(root, "ic"),
        &sample_nns_node_provider_list_report(),
    );
    write_json_cache(
        &nns_node_operator_cache_path(root, "ic"),
        &sample_nns_node_operator_list_report(),
    );
}

#[cfg(feature = "host")]
fn assert_public_nns_node_host_api(root: &Path) {
    let cache = NnsNodeCacheRequest::new(root, "ic");
    let request = NnsNodeListRequest {
        cache: cache.clone(),
        source_endpoint: DEFAULT_NNS_NODE_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        filters: NnsNodeListFilters::default(),
    };
    let list = build_nns_node_list_report(&request).expect("build node list from cache");
    let info = build_nns_node_info_report(&NnsNodeInfoRequest {
        cache: cache.clone(),
        source_endpoint: DEFAULT_NNS_NODE_SOURCE_ENDPOINT.to_string(),
        input: sample_nns_node_row().node_principal,
        now_unix_secs: 1_700_000_000,
    })
    .expect("build node info from cache");
    let refresh = sample_nns_node_refresh_report(root);
    let refresh_request = NnsNodeRefreshRequest {
        cache,
        source_endpoint: DEFAULT_NNS_NODE_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        lock_stale_after_seconds: DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS,
        dry_run: true,
        output_path: None,
    };

    assert_eq!(list.node_count, 1);
    assert_eq!(info.node_principal, sample_nns_node_row().node_principal);
    assert!(nns_node_cache_path(root, "ic").is_file());
    assert!(nns_node_refresh_lock_path(root, "ic").ends_with("refresh.lock"));
    assert!(nns_node_refresh_report_text(&refresh).contains("node_count: 1"));
    assert!(refresh_api_accepts_public_types(
        refresh_nns_node_report,
        &refresh_request
    ));
}

#[cfg(feature = "host")]
fn assert_public_nns_data_center_host_api(root: &Path) {
    let cache = NnsDataCenterCacheRequest::new(root, "ic");
    let request = NnsDataCenterListRequest {
        cache: cache.clone(),
        source_endpoint: DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
    };
    let list =
        build_nns_data_center_list_report(&request).expect("build data-center list from cache");
    let info = build_nns_data_center_info_report(&NnsDataCenterInfoRequest {
        cache: cache.clone(),
        source_endpoint: DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT.to_string(),
        input: sample_nns_data_center_row().data_center_id,
        now_unix_secs: 1_700_000_000,
    })
    .expect("build data-center info from cache");
    let refresh = sample_nns_data_center_refresh_report(root);
    let refresh_request = NnsDataCenterRefreshRequest {
        cache,
        source_endpoint: DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        lock_stale_after_seconds: DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS,
        dry_run: true,
        output_path: None,
    };

    assert_eq!(list.data_center_count, 1);
    assert_eq!(
        info.data_center_id,
        sample_nns_data_center_row().data_center_id
    );
    assert!(nns_data_center_cache_path(root, "ic").is_file());
    assert!(nns_data_center_refresh_lock_path(root, "ic").ends_with("refresh.lock"));
    assert!(nns_data_center_refresh_report_text(&refresh).contains("data_center_count: 1"));
    assert!(refresh_api_accepts_public_types(
        refresh_nns_data_center_report,
        &refresh_request
    ));
}

#[cfg(feature = "host")]
fn assert_public_nns_node_provider_host_api(root: &Path) {
    let cache = NnsNodeProviderCacheRequest::new(root, "ic");
    let request = NnsNodeProviderListRequest {
        cache: cache.clone(),
        source_endpoint: DEFAULT_NNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
    };
    let list =
        build_nns_node_provider_list_report(&request).expect("build node-provider list from cache");
    let info = build_nns_node_provider_info_report(&NnsNodeProviderInfoRequest {
        cache: cache.clone(),
        source_endpoint: DEFAULT_NNS_SOURCE_ENDPOINT.to_string(),
        input: sample_nns_node_provider_row().node_provider_principal,
        now_unix_secs: 1_700_000_000,
    })
    .expect("build node-provider info from cache");
    let refresh = sample_nns_node_provider_refresh_report(root);
    let refresh_request = NnsNodeProviderRefreshRequest {
        cache,
        source_endpoint: DEFAULT_NNS_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        lock_stale_after_seconds: DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS,
        dry_run: true,
        output_path: None,
    };

    assert_eq!(list.node_provider_count, 1);
    assert_eq!(
        info.node_provider_principal,
        sample_nns_node_provider_row().node_provider_principal
    );
    assert!(nns_node_provider_cache_path(root, "ic").is_file());
    assert!(nns_node_provider_refresh_lock_path(root, "ic").ends_with("refresh.lock"));
    assert!(nns_node_provider_refresh_report_text(&refresh).contains("node_provider_count: 1"));
    assert!(refresh_api_accepts_public_types(
        refresh_nns_node_provider_report,
        &refresh_request
    ));
}

#[cfg(feature = "host")]
fn assert_public_nns_node_operator_host_api(root: &Path) {
    let cache = NnsNodeOperatorCacheRequest::new(root, "ic");
    let request = NnsNodeOperatorListRequest {
        cache: cache.clone(),
        source_endpoint: DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
    };
    let list =
        build_nns_node_operator_list_report(&request).expect("build node-operator list from cache");
    let info = build_nns_node_operator_info_report(&NnsNodeOperatorInfoRequest {
        cache: cache.clone(),
        source_endpoint: DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT.to_string(),
        input: sample_nns_node_operator_row().node_operator_principal,
        now_unix_secs: 1_700_000_000,
    })
    .expect("build node-operator info from cache");
    let refresh = sample_nns_node_operator_refresh_report(root);
    let refresh_request = NnsNodeOperatorRefreshRequest {
        cache,
        source_endpoint: DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        lock_stale_after_seconds: DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS,
        dry_run: true,
        output_path: None,
    };

    assert_eq!(list.node_operator_count, 1);
    assert_eq!(
        info.node_operator_principal,
        sample_nns_node_operator_row().node_operator_principal
    );
    assert!(nns_node_operator_cache_path(root, "ic").is_file());
    assert!(nns_node_operator_refresh_lock_path(root, "ic").ends_with("refresh.lock"));
    assert!(nns_node_operator_refresh_report_text(&refresh).contains("node_operator_count: 1"));
    assert!(refresh_api_accepts_public_types(
        refresh_nns_node_operator_report,
        &refresh_request
    ));
}

#[cfg(feature = "host")]
fn write_json_cache<T>(path: &Path, value: &T)
where
    T: Serialize,
{
    fs::create_dir_all(path.parent().expect("cache parent")).expect("create cache parent");
    fs::write(
        path,
        serde_json::to_string_pretty(value).expect("serialize fixture cache"),
    )
    .expect("write fixture cache");
}

#[cfg(feature = "host")]
#[must_use]
fn refresh_api_accepts_public_types<Request, Report, Error>(
    _refresh: RefreshFn<Request, Report, Error>,
    request: &Request,
) -> bool {
    std::mem::size_of_val(request) > 0
}

#[cfg(feature = "host")]
#[must_use]
fn temp_root(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("ic-query-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    path
}

#[cfg(feature = "host")]
fn sample_nns_node_list_report() -> NnsNodeListReport {
    let node = sample_nns_node_row();
    NnsNodeListReport {
        schema_version: 1,
        network: "ic".to_string(),
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: DEFAULT_NNS_NODE_SOURCE_ENDPOINT.to_string(),
        fetched_by: "ic-query".to_string(),
        node_count: 1,
        nodes: vec![node],
    }
}

#[cfg(feature = "host")]
fn sample_nns_data_center_list_report() -> NnsDataCenterListReport {
    let data_center = sample_nns_data_center_row();
    NnsDataCenterListReport {
        schema_version: 1,
        network: "ic".to_string(),
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT.to_string(),
        fetched_by: "ic-query".to_string(),
        data_center_count: 1,
        data_centers: vec![data_center],
    }
}

#[cfg(feature = "host")]
fn sample_nns_node_provider_list_report() -> NnsNodeProviderListReport {
    let provider = sample_nns_node_provider_row();
    NnsNodeProviderListReport {
        schema_version: 1,
        network: "ic".to_string(),
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: DEFAULT_NNS_SOURCE_ENDPOINT.to_string(),
        fetched_by: "ic-query".to_string(),
        node_provider_count: 1,
        node_providers: vec![provider],
    }
}

#[cfg(feature = "host")]
fn sample_nns_node_operator_list_report() -> NnsNodeOperatorListReport {
    let operator = sample_nns_node_operator_row();
    NnsNodeOperatorListReport {
        schema_version: 1,
        network: "ic".to_string(),
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT.to_string(),
        fetched_by: "ic-query".to_string(),
        node_operator_count: 1,
        node_operators: vec![operator],
    }
}

#[cfg(feature = "host")]
fn sample_nns_node_refresh_report(root: &Path) -> NnsNodeRefreshReport {
    NnsNodeRefreshReport {
        schema_version: 1,
        network: "ic".to_string(),
        cache_path: nns_node_cache_path(root, "ic").display().to_string(),
        refresh_lock_path: nns_node_refresh_lock_path(root, "ic").display().to_string(),
        output_path: None,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: DEFAULT_NNS_NODE_SOURCE_ENDPOINT.to_string(),
        fetched_by: "ic-query".to_string(),
        dry_run: true,
        wrote_cache: false,
        replaced_existing_cache: true,
        node_count: 1,
    }
}

#[cfg(feature = "host")]
fn sample_nns_data_center_refresh_report(root: &Path) -> NnsDataCenterRefreshReport {
    NnsDataCenterRefreshReport {
        schema_version: 1,
        network: "ic".to_string(),
        cache_path: nns_data_center_cache_path(root, "ic").display().to_string(),
        refresh_lock_path: nns_data_center_refresh_lock_path(root, "ic")
            .display()
            .to_string(),
        output_path: None,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT.to_string(),
        fetched_by: "ic-query".to_string(),
        dry_run: true,
        wrote_cache: false,
        replaced_existing_cache: true,
        data_center_count: 1,
    }
}

#[cfg(feature = "host")]
fn sample_nns_node_provider_refresh_report(root: &Path) -> NnsNodeProviderRefreshReport {
    NnsNodeProviderRefreshReport {
        schema_version: 1,
        network: "ic".to_string(),
        cache_path: nns_node_provider_cache_path(root, "ic")
            .display()
            .to_string(),
        refresh_lock_path: nns_node_provider_refresh_lock_path(root, "ic")
            .display()
            .to_string(),
        output_path: None,
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: DEFAULT_NNS_SOURCE_ENDPOINT.to_string(),
        fetched_by: "ic-query".to_string(),
        dry_run: true,
        wrote_cache: false,
        replaced_existing_cache: true,
        node_provider_count: 1,
    }
}

#[cfg(feature = "host")]
fn sample_nns_node_operator_refresh_report(root: &Path) -> NnsNodeOperatorRefreshReport {
    NnsNodeOperatorRefreshReport {
        schema_version: 1,
        network: "ic".to_string(),
        cache_path: nns_node_operator_cache_path(root, "ic")
            .display()
            .to_string(),
        refresh_lock_path: nns_node_operator_refresh_lock_path(root, "ic")
            .display()
            .to_string(),
        output_path: None,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT.to_string(),
        fetched_by: "ic-query".to_string(),
        dry_run: true,
        wrote_cache: false,
        replaced_existing_cache: true,
        node_operator_count: 1,
    }
}

fn sample_nns_node_row() -> NnsNodeRow {
    NnsNodeRow {
        node_principal: "zh3jp-xqaaa-aaaar-qaada-cai".to_string(),
        node_operator_principal: "qoctq-giaaa-aaaar-qaada-cai".to_string(),
        node_provider_principal: "w6gnz-6qaaa-aaaar-qaada-cai".to_string(),
        subnet_principal: "tdb26-jop6g-7sc54-foywl".to_string(),
        subnet_kind: NNS_NODE_SUBNET_KIND_APPLICATION.to_string(),
        data_center_id: "zh1".to_string(),
    }
}

fn sample_nns_data_center_row() -> NnsDataCenterRow {
    NnsDataCenterRow {
        data_center_id: "zh1".to_string(),
        region: "Zurich".to_string(),
        owner: "Example DC Owner".to_string(),
        latitude: Some(47.37),
        longitude: Some(8.54),
        node_operator_count: 2,
        node_provider_count: 3,
        node_count: 12,
    }
}

fn sample_nns_node_provider_row() -> NnsNodeProviderRow {
    NnsNodeProviderRow {
        node_provider_principal: "w6gnz-6qaaa-aaaar-qaada-cai".to_string(),
        name: Some("Example Provider".to_string()),
        node_count: Some(12),
        reward_account_hex: Some("deadbeef".to_string()),
    }
}

fn sample_nns_node_operator_row() -> NnsNodeOperatorRow {
    NnsNodeOperatorRow {
        node_operator_principal: "tdb26-jop6g-7sc54-foywl".to_string(),
        node_provider_principal: "w6gnz-6qaaa-aaaar-qaada-cai".to_string(),
        node_allowance: 28,
        data_center_id: "zh1".to_string(),
        node_count: Some(12),
    }
}

fn sample_nns_proposal_row() -> NnsProposalRow {
    NnsProposalRow {
        proposal_id: Some(132_411),
        proposer_neuron_id: Some(12_345),
        topic: 4,
        topic_text: "governance".to_string(),
        status: 4,
        status_text: "executed".to_string(),
        reward_status: 3,
        reward_status_text: "settled".to_string(),
        title: Some("Upgrade subnet".to_string()),
        summary: "Upgrade subnet replica version.".to_string(),
        url: "https://dashboard.internetcomputer.org/proposal/132411".to_string(),
        action_text: Some("execute-nns-function".to_string()),
        reject_cost_e8s: 100_000_000,
        proposal_timestamp_seconds: 1_700_000_000,
        proposed_at: "2023-11-14T22:13:20Z".to_string(),
        deadline_timestamp_seconds: Some(1_700_086_400),
        deadline_at: Some("2023-11-15T22:13:20Z".to_string()),
        decided_timestamp_seconds: 1_700_010_000,
        decided_at: Some("2023-11-15T01:00:00Z".to_string()),
        executed_timestamp_seconds: 1_700_020_000,
        executed_at: Some("2023-11-15T03:46:40Z".to_string()),
        failed_timestamp_seconds: 0,
        failed_at: None,
        reward_event_round: 42,
        total_potential_voting_power: Some(1_000_000_000),
        latest_tally: Some(NnsProposalTally {
            timestamp_seconds: 1_700_010_000,
            yes: 900_000_000,
            no: 100_000_000,
            total: 1_000_000_000,
        }),
        ballot_count: 1,
        ballots: vec![NnsProposalBallotRow {
            neuron_id: 12_345,
            vote: 1,
            vote_text: "yes".to_string(),
            voting_power: 100_000_000,
        }],
    }
}

fn sample_topology_registry_version_row() -> NnsTopologyRegistryVersionRow {
    NnsTopologyRegistryVersionRow {
        source: "subnet_catalog".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        stale: Some(false),
    }
}
