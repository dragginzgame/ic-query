use ic_query::nns::data_center::{
    NnsDataCenterCacheRequest, NnsDataCenterInfoReport, NnsDataCenterInfoRequest,
    NnsDataCenterListReport, NnsDataCenterListRequest, NnsDataCenterRow,
    nns_data_center_info_report_text, nns_data_center_list_report_text,
    nns_data_center_list_report_verbose_text,
};
use ic_query::nns::node::{
    NNS_NODE_SUBNET_KIND_APPLICATION, NnsNodeCacheRequest, NnsNodeInfoReport, NnsNodeInfoRequest,
    NnsNodeListFilters, NnsNodeListReport, NnsNodeListRequest, NnsNodeRow,
    nns_node_info_report_text, nns_node_list_report_text, nns_node_list_report_verbose_text,
};
use ic_query::nns::node_operator::{
    NnsNodeOperatorCacheRequest, NnsNodeOperatorInfoReport, NnsNodeOperatorInfoRequest,
    NnsNodeOperatorListReport, NnsNodeOperatorListRequest, NnsNodeOperatorRow,
    nns_node_operator_info_report_text, nns_node_operator_list_report_text,
    nns_node_operator_list_report_verbose_text,
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
