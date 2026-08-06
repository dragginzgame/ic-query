#[cfg(feature = "nns-host")]
use ic_query::nns::data_center::{
    DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS, DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
    NnsDataCenterHostError, NnsDataCenterRefreshReport, NnsDataCenterSource,
    build_nns_data_center_info_report, build_nns_data_center_info_report_with_source,
    build_nns_data_center_list_report, build_nns_data_center_list_report_with_source,
    nns_data_center_cache_path, nns_data_center_refresh_lock_path,
    nns_data_center_refresh_report_text, refresh_nns_data_center_report,
    refresh_nns_data_center_report_with_source,
};
use ic_query::nns::data_center::{
    NnsDataCenterInfoReport, NnsDataCenterListReport, NnsDataCenterRow,
    nns_data_center_info_report_text, nns_data_center_list_report_text,
    nns_data_center_list_report_verbose_text,
};
use ic_query::nns::governance::{
    DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT, NnsGovernanceMaturityModulation,
    NnsGovernanceMaturityModulationReport, NnsGovernanceReportContext,
    nns_governance_maturity_modulation_report_text,
};
#[cfg(feature = "nns-host")]
use ic_query::nns::governance::{
    NnsGovernanceEconomics, NnsGovernanceHostError, NnsGovernanceSource,
    build_nns_governance_economics_report_with_source,
};
#[cfg(feature = "nns-host")]
use ic_query::nns::neuron::{
    DEFAULT_NNS_NEURON_SOURCE_ENDPOINT, NnsNeuronHostError, NnsNeuronInfoRequest, NnsNeuronPage,
    NnsNeuronSource, build_nns_neuron_cache_status_report,
    build_nns_neuron_info_report_with_source, build_nns_neuron_list_report_with_source,
    nns_neuron_cache_path, nns_neuron_refresh_attempt_path, nns_neuron_refresh_lock_path,
};
use ic_query::nns::neuron::{
    NNS_NEURON_MAX_PAGE_SIZE, NnsKnownNeuronData, NnsNeuronListRequest, NnsNeuronRow,
    NnsNeuronState, NnsNeuronType, NnsNeuronVisibility, NnsNeuronVote, nns_neuron_info_report_text,
    nns_neuron_list_report_text,
};
#[cfg(feature = "nns-host")]
use ic_query::nns::node::{
    DEFAULT_NNS_NODE_SOURCE_ENDPOINT, DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS, NnsNodeHostError,
    NnsNodeRefreshReport, NnsNodeSource, build_nns_node_info_report,
    build_nns_node_info_report_with_source, build_nns_node_list_report,
    build_nns_node_list_report_with_source, nns_node_cache_path, nns_node_refresh_lock_path,
    nns_node_refresh_report_text, refresh_nns_node_report, refresh_nns_node_report_with_source,
};
use ic_query::nns::node::{
    NnsNodeInfoReport, NnsNodeListReport, NnsNodeListRequest, NnsNodeRow,
    nns_node_info_report_text, nns_node_list_report_text, nns_node_list_report_verbose_text,
};
#[cfg(feature = "nns-host")]
use ic_query::nns::node_operator::{
    DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT, DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS,
    NnsNodeOperatorHostError, NnsNodeOperatorRefreshReport, NnsNodeOperatorSource,
    build_nns_node_operator_info_report, build_nns_node_operator_info_report_with_source,
    build_nns_node_operator_list_report, build_nns_node_operator_list_report_with_source,
    nns_node_operator_cache_path, nns_node_operator_refresh_lock_path,
    nns_node_operator_refresh_report_text, refresh_nns_node_operator_report,
    refresh_nns_node_operator_report_with_source,
};
use ic_query::nns::node_operator::{
    NnsNodeOperatorInfoReport, NnsNodeOperatorListReport, NnsNodeOperatorRow,
    nns_node_operator_info_report_text, nns_node_operator_list_report_text,
    nns_node_operator_list_report_verbose_text,
};
#[cfg(feature = "nns-host")]
use ic_query::nns::node_provider::{
    DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT, DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS,
    NnsNodeProviderHostError, NnsNodeProviderRefreshReport, NnsNodeProviderSource,
    build_nns_node_provider_info_report, build_nns_node_provider_info_report_with_source,
    build_nns_node_provider_list_report, build_nns_node_provider_list_report_with_source,
    nns_node_provider_cache_path, nns_node_provider_refresh_lock_path,
    nns_node_provider_refresh_report_text, refresh_nns_node_provider_report,
    refresh_nns_node_provider_report_with_source,
};
use ic_query::nns::node_provider::{
    NnsNodeProviderInfoReport, NnsNodeProviderListReport, NnsNodeProviderRow,
    nns_node_provider_info_report_text, nns_node_provider_list_report_text,
    nns_node_provider_list_report_verbose_text,
};
#[cfg(feature = "nns-host")]
use ic_query::nns::proposals::{
    DEFAULT_NNS_PROPOSAL_REFRESH_LOCK_STALE_SECONDS, DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT,
    NnsProposalHostError, NnsProposalRefreshReport, NnsProposalSource,
    build_nns_proposal_cache_list_report, build_nns_proposal_cache_status_report,
    build_nns_proposal_list_report, build_nns_proposal_list_report_from_cache,
    build_nns_proposal_list_report_with_source, build_nns_proposal_report,
    build_nns_proposal_report_from_cache, build_nns_proposal_report_with_source,
    nns_proposal_cache_list_report_text, nns_proposal_cache_path, nns_proposal_cache_root,
    nns_proposal_cache_status_report_text, nns_proposal_refresh_attempt_path,
    nns_proposal_refresh_lock_path, nns_proposal_refresh_report_text, refresh_nns_proposal_cache,
    refresh_nns_proposal_cache_with_source,
};
use ic_query::nns::proposals::{
    NnsProposalBallotRow, NnsProposalListReport, NnsProposalListRequest, NnsProposalListSort,
    NnsProposalReport, NnsProposalRequest, NnsProposalRewardStatus, NnsProposalRewardStatusFilter,
    NnsProposalRow, NnsProposalSortDirection, NnsProposalStatus, NnsProposalStatusFilter,
    NnsProposalTally, NnsProposalTopic, NnsProposalTopicFilter, NnsProposalVote,
    nns_proposal_list_report_text, nns_proposal_report_text,
};
use ic_query::nns::registry::{
    NnsCertifiedRegistryDeltaBatchReport, NnsCertifiedRegistryDeltaBatchRequest,
    NnsCertifiedRegistryDeltaLimits, NnsCertifiedRegistryDeltaVersion,
    NnsCertifiedRegistryMutation, NnsCertifiedRegistryMutationKind,
    NnsCertifiedRegistryValueEncoding, NnsRegistryCertification, NnsRegistryVersionReport,
    NnsRegistryVersionRequest, nns_registry_version_report_text,
};
#[cfg(feature = "nns-host")]
use ic_query::nns::registry::{
    NnsCertifiedRegistryDeltaSource, NnsCertifiedRegistryDeltaSourceFuture, NnsRegistryHostError,
    NnsRegistryReplayError, NnsRegistryReplayLimits, NnsRegistryReplaySession,
    NnsRegistryReplaySessionLimits, NnsRegistryReplayState, NnsRegistrySource,
    NnsRegistryVersionData, apply_nns_certified_registry_delta_batch,
    build_nns_registry_version_report_with_source,
    fetch_nns_certified_registry_delta_batch_with_source_async,
    nns_certified_registry_delta_limits,
};
#[cfg(feature = "nns-host")]
use ic_query::nns::topology::{
    DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT, NnsTopologyHostError, NnsTopologyRefreshSource,
    NnsTopologyRefreshSourceRequest, NnsTopologySource, NnsTopologySourceRequest,
    build_nns_topology_capacity_report_with_source, build_nns_topology_check_report_with_source,
    build_nns_topology_coverage_report_with_source, build_nns_topology_gaps_report_with_source,
    build_nns_topology_providers_report_with_source, build_nns_topology_regions_report_with_source,
    build_nns_topology_summary_report_with_source, build_nns_topology_versions_report_with_source,
    refresh_nns_topology_report_with_source,
};
use ic_query::nns::topology::{
    NnsTopologyAssessmentStatus, NnsTopologyCapacityReport, NnsTopologyCapacityRow,
    NnsTopologyCapacityStatus, NnsTopologyCheckReport, NnsTopologyCheckRow,
    NnsTopologyCoverageReport, NnsTopologyGapRelationKind, NnsTopologyGapRow,
    NnsTopologyGapSubjectKind, NnsTopologyGapsReport, NnsTopologyProviderRow,
    NnsTopologyProviderStatus, NnsTopologyProvidersReport, NnsTopologyReadRequest,
    NnsTopologyRefreshReport, NnsTopologyRefreshRequest, NnsTopologyRefreshRow,
    NnsTopologyRegionRow, NnsTopologyRegionsReport, NnsTopologyRegistryVersionRow,
    NnsTopologySummaryReport, NnsTopologyVersionsReport, nns_topology_capacity_report_text,
    nns_topology_check_report_text, nns_topology_coverage_report_text,
    nns_topology_gaps_report_text, nns_topology_providers_report_text,
    nns_topology_refresh_report_text, nns_topology_regions_report_text,
    nns_topology_summary_report_text, nns_topology_versions_report_text,
};
#[cfg(feature = "nns-host")]
use ic_query::nns::{
    NnsGovernanceCacheRequest, NnsGovernanceQueryError, NnsGovernanceRefreshRequest,
    NnsInventoryRefreshRequest, NnsSourceRequest,
};
use ic_query::nns::{NnsInventoryCacheRequest, NnsInventoryInfoRequest, NnsInventoryListRequest};
use ic_query::report::{ReportDataSource, ReportResultScope};
use ic_query::subnet_catalog::SubnetKind;
#[cfg(feature = "nns-host")]
use ic_query::subnet_catalog::{
    CacheDisposition, CatalogAssurance, ClassificationSource, GeographicScope,
    SubnetCatalogListReport, SubnetCatalogRefreshReport, SubnetCatalogSubnetRow,
    SubnetSpecialization,
};
#[cfg(feature = "nns-host")]
use serde::Serialize;
#[cfg(feature = "nns-host")]
use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(feature = "nns-host")]
#[test]
fn public_nns_governance_collection_contracts_are_shared() {
    let cache = NnsGovernanceCacheRequest::new("/tmp/ic-query-governance-contract", "ic");
    let refresh = NnsGovernanceRefreshRequest::new(
        cache.cache_root(),
        &cache.network,
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT,
        1_700_000_000,
        100,
    )
    .with_max_pages(Some(2));
    assert_eq!(refresh.max_pages, Some(2));

    let query_error = NnsGovernanceQueryError::AgentCall {
        method: "list_proposals",
        reason: "fixture failure".to_string(),
    };
    assert!(matches!(
        NnsProposalHostError::from(query_error.clone()),
        NnsProposalHostError::GovernanceQuery(NnsGovernanceQueryError::AgentCall {
            method: "list_proposals",
            ..
        })
    ));
    assert!(matches!(
        NnsNeuronHostError::from(query_error),
        NnsNeuronHostError::GovernanceQuery(NnsGovernanceQueryError::AgentCall {
            method: "list_proposals",
            ..
        })
    ));
}

#[test]
fn public_nns_governance_report_api_is_constructible_and_renderable() {
    let report = NnsGovernanceMaturityModulationReport {
        context: NnsGovernanceReportContext {
            schema_version: 1,
            network: "ic".to_string(),
            governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
            fetched_at: "2026-07-30T00:00:00Z".to_string(),
            source_endpoint: DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT.to_string(),
            fetched_by: "fixture".to_string(),
        },
        maturity_modulation: Some(NnsGovernanceMaturityModulation {
            current_value_permyriad: Some(-125),
            updated_at_timestamp_seconds: Some(1_700_000_000),
        }),
    };

    let text = nns_governance_maturity_modulation_report_text(&report);
    assert!(text.contains("current_value_permyriad: -125"));
    let json = serde_json::to_value(report).expect("serialize public report");
    assert_eq!(json["network"], "ic");
    assert_eq!(json["maturity_modulation"]["current_value_permyriad"], -125);
}

#[cfg(feature = "nns-host")]
#[test]
fn public_nns_governance_host_api_accepts_custom_source() {
    let request = NnsSourceRequest::from_unix_secs(
        "ic",
        DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT,
        1_700_000_000,
        "fixture",
    );
    let report =
        build_nns_governance_economics_report_with_source(&request, &FixtureGovernanceSource)
            .expect("custom Governance source");

    assert_eq!(report.economics.transaction_fee_e8s, 10_000);
    assert_eq!(report.context.fetched_at, "2023-11-14T22:13:20Z");
}

#[cfg(feature = "nns-host")]
struct FixtureGovernanceSource;

#[cfg(feature = "nns-host")]
impl NnsGovernanceSource for FixtureGovernanceSource {
    fn fetch_economics(
        &self,
        _request: &NnsSourceRequest,
    ) -> Result<NnsGovernanceEconomics, NnsGovernanceHostError> {
        Ok(NnsGovernanceEconomics {
            neuron_minimum_stake_e8s: 100_000_000,
            max_proposals_to_keep_per_topic: 100,
            neuron_management_fee_per_proposal_e8s: 10_000,
            reject_cost_e8s: 1_000_000_000,
            transaction_fee_e8s: 10_000,
            neuron_spawn_dissolve_delay_seconds: 604_800,
            minimum_icp_xdr_rate: 100,
            maximum_node_provider_rewards_e8s: 200_000_000,
            neurons_fund_economics: None,
            voting_power_economics: None,
        })
    }

    fn fetch_metrics(
        &self,
        _request: &NnsSourceRequest,
    ) -> Result<ic_query::nns::governance::NnsGovernanceMetrics, NnsGovernanceHostError> {
        unreachable!("not used by this public API fixture")
    }

    fn fetch_reward_event(
        &self,
        _request: &NnsSourceRequest,
    ) -> Result<ic_query::nns::governance::NnsGovernanceRewardEvent, NnsGovernanceHostError> {
        unreachable!("not used by this public API fixture")
    }

    fn fetch_maturity_modulation(
        &self,
        _request: &NnsSourceRequest,
    ) -> Result<Option<NnsGovernanceMaturityModulation>, NnsGovernanceHostError> {
        unreachable!("not used by this public API fixture")
    }
}

#[test]
fn public_nns_neuron_api_is_constructible_and_renderable() {
    let request = NnsNeuronListRequest::new("ic", "https://icp-api.io", 1_700_000_000, 25)
        .with_exclusive_start_neuron_id(10)
        .with_verbose(true);
    assert_eq!(request.limit, 25);
    assert_eq!(request.exclusive_start_neuron_id, Some(10));
    assert_eq!(NNS_NEURON_MAX_PAGE_SIZE, 300);

    let row = sample_public_neuron(11);
    let report = ic_query::nns::neuron::NnsNeuronListReport {
        schema_version: 2,
        network: request.network,
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        cache_path: None,
        from_cache: false,
        requested_limit: request.limit,
        exclusive_start_neuron_id: request.exclusive_start_neuron_id,
        next_start_neuron_id: None,
        total_neuron_count: None,
        point_in_time_guaranteed: false,
        returned_neuron_count: 1,
        verbose: request.verbose,
        neurons: vec![row.clone()],
    };
    assert!(nns_neuron_list_report_text(&report).contains("Neuron 11"));
    let report_json = serde_json::to_value(&report).expect("serialize NNS neuron list report");
    assert_eq!(report_json["neurons"][0]["state_text"], "not-dissolving");
    assert_eq!(report_json["neurons"][0]["visibility_text"], "public");
    assert_eq!(report_json["neurons"][0]["neuron_type_text"], "unknown");
    assert_eq!(
        serde_json::to_value(NnsNeuronVote::Unknown(99)).expect("serialize unknown neuron vote"),
        "unknown(99)"
    );

    let info = ic_query::nns::neuron::NnsNeuronInfoReport {
        schema_version: 1,
        network: "ic".to_string(),
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "ic-query".to_string(),
        cache_path: None,
        from_cache: false,
        verbose: true,
        neuron: row,
    };
    assert!(nns_neuron_info_report_text(&info).contains("neuron_id: 11"));
}

#[cfg(feature = "nns-host")]
#[test]
fn public_nns_neuron_host_api_accepts_custom_source_and_cache_requests() {
    let list_request =
        NnsNeuronListRequest::new("ic", DEFAULT_NNS_NEURON_SOURCE_ENDPOINT, 1_700_000_000, 1);
    let list = build_nns_neuron_list_report_with_source(&list_request, &FixtureNnsNeuronSource)
        .expect("custom neuron source");
    assert_eq!(list.neurons[0].neuron_id, 7);

    let info_request =
        NnsNeuronInfoRequest::new("ic", DEFAULT_NNS_NEURON_SOURCE_ENDPOINT, 1_700_000_000, 7);
    let info = build_nns_neuron_info_report_with_source(&info_request, &FixtureNnsNeuronSource)
        .expect("custom neuron detail");
    assert_eq!(info.neuron.neuron_id, 7);

    let root = PathBuf::from("/tmp/ic-query-public-neuron-api");
    let refresh = NnsGovernanceRefreshRequest::new(
        &root,
        "ic",
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        1_700_000_000,
        1,
    );
    assert_eq!(refresh.page_size, 1);
    assert!(nns_neuron_cache_path(&root, "ic").ends_with("full.json"));
    assert!(nns_neuron_refresh_lock_path(&root, "ic").ends_with("full.refresh.lock"));
    assert!(nns_neuron_refresh_attempt_path(&root, "ic").ends_with("full.refresh-attempt.json"));

    let status = build_nns_neuron_cache_status_report(&NnsGovernanceCacheRequest::new(root, "ic"))
        .expect("missing cache status");
    assert!(!status.found);
}

#[cfg(feature = "nns-host")]
struct FixtureNnsNeuronSource;

#[cfg(feature = "nns-host")]
impl NnsNeuronSource for FixtureNnsNeuronSource {
    fn fetch_neuron_page(
        &self,
        _request: &NnsSourceRequest,
        _exclusive_start_neuron_id: Option<u64>,
        page_size: u32,
    ) -> Result<NnsNeuronPage, NnsNeuronHostError> {
        let neurons = vec![sample_public_neuron(7)];
        Ok(NnsNeuronPage {
            next_start_neuron_id: (page_size == 1).then_some(7),
            neurons,
        })
    }

    fn fetch_neuron(
        &self,
        _request: &NnsSourceRequest,
        neuron_id: u64,
    ) -> Result<NnsNeuronRow, NnsNeuronHostError> {
        Ok(sample_public_neuron(neuron_id))
    }
}

fn sample_public_neuron(neuron_id: u64) -> NnsNeuronRow {
    NnsNeuronRow {
        neuron_id,
        state: 1,
        state_text: NnsNeuronState::NotDissolving,
        visibility: Some(2),
        visibility_text: NnsNeuronVisibility::Public,
        neuron_type: None,
        neuron_type_text: NnsNeuronType::Unknown,
        stake_e8s: 100_000_000,
        staked_maturity_e8s_equivalent: None,
        dissolve_delay_seconds: 31_536_000,
        age_seconds: 86_400,
        created_timestamp_seconds: 1_600_000_000,
        retrieved_at_timestamp_seconds: 1_700_000_000,
        voting_power: 100_000_000,
        deciding_voting_power: Some(100_000_000),
        potential_voting_power: Some(100_000_000),
        voting_power_refreshed_timestamp_seconds: Some(1_699_999_000),
        joined_community_fund_timestamp_seconds: None,
        eight_year_gang_bonus_base_e8s: None,
        known_neuron_data: Some(NnsKnownNeuronData {
            name: format!("Neuron {neuron_id}"),
            description: None,
            links: Vec::new(),
        }),
        recent_ballots: Vec::new(),
    }
}

#[test]
fn public_nns_registry_api_is_constructible_and_renderable() {
    let request = NnsRegistryVersionRequest::new("ic", "https://icp-api.io", 1_700_000_000);

    assert_eq!(request.network, "ic");

    let report = NnsRegistryVersionReport {
        schema_version: 2,
        network: request.network,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        certification: public_registry_certification(),
    };

    let text = nns_registry_version_report_text(&report);

    assert!(text.contains("network: ic"));
    assert!(text.contains("registry_version: 42"));
}

#[test]
fn public_certified_registry_delta_models_preserve_raw_evidence() {
    let request =
        NnsCertifiedRegistryDeltaBatchRequest::new("ic", "https://icp-api.io", 41, 1_700_000_000);
    let report = public_certified_delta_report(&request);

    let json = serde_json::to_value(report).expect("serialize certified delta report");

    assert_eq!(json["requested_version"], 41);
    assert_eq!(json["versions"][0]["mutations"][0]["mutation_type"], 4);
    assert_eq!(
        json["versions"][0]["mutations"][0]["mutation_kind"],
        "upsert"
    );
    assert_eq!(json["versions"][0]["mutations"][0]["key_hex"], "61");
    assert_eq!(
        json["versions"][0]["mutations"][0]["value_encoding"],
        "inline"
    );
    assert_eq!(json["chunk_reference_count"], 0);
    assert_eq!(json["query_call_count"], 1);
}

#[cfg(feature = "nns-host")]
#[test]
fn public_nns_registry_host_api_accepts_custom_source_adapter() {
    let request = NnsRegistryVersionRequest::new("ic", "https://icp-api.io", 1_700_000_000);
    let report = build_nns_registry_version_report_with_source(&request, &FixtureNnsRegistrySource)
        .expect("registry version report");

    assert_eq!(report.network, "ic");
    assert_eq!(report.registry_canister_id, "rwlgt-iiaaa-aaaaa-aaaaa-cai");
    assert_eq!(report.registry_version, 42);
    assert_eq!(report.source_endpoint, "https://icp-api.io");
}

#[cfg(feature = "nns-host")]
struct FixtureNnsRegistrySource;

#[cfg(feature = "nns-host")]
impl NnsRegistrySource for FixtureNnsRegistrySource {
    fn fetch_registry_version(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsRegistryVersionData, NnsRegistryHostError> {
        assert_eq!(request.endpoint, "https://icp-api.io");
        assert_eq!(request.fetched_by, "ic-query");
        assert!(!request.fetched_at.is_empty());

        Ok(NnsRegistryVersionData {
            network: "ic".to_string(),
            registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
            registry_version: 42,
            fetched_at: request.fetched_at.clone(),
            fetched_by: request.fetched_by.clone(),
            source_endpoint: request.endpoint.clone(),
            certification: public_registry_certification(),
        })
    }
}

#[cfg(feature = "nns-host")]
struct FixtureCertifiedRegistryDeltaSource;

#[cfg(feature = "nns-host")]
impl NnsCertifiedRegistryDeltaSource for FixtureCertifiedRegistryDeltaSource {
    fn fetch_certified_registry_delta_batch<'a>(
        &'a self,
        request: &'a NnsCertifiedRegistryDeltaBatchRequest,
    ) -> NnsCertifiedRegistryDeltaSourceFuture<'a> {
        Box::pin(async move { Ok(public_certified_delta_report(request)) })
    }
}

#[cfg(feature = "nns-host")]
#[test]
fn public_certified_registry_delta_async_api_accepts_custom_sources() {
    let request =
        NnsCertifiedRegistryDeltaBatchRequest::new("ic", "https://icp-api.io", 41, 1_700_000_000);
    let report =
        futures::executor::block_on(fetch_nns_certified_registry_delta_batch_with_source_async(
            &request,
            &FixtureCertifiedRegistryDeltaSource,
        ))
        .expect("public certified delta API");

    assert_eq!(report.first_version, Some(42));
}

#[cfg(feature = "nns-host")]
#[test]
fn public_certified_registry_replay_api_is_bounded_and_version_checked() {
    let request =
        NnsCertifiedRegistryDeltaBatchRequest::new("ic", "https://icp-api.io", 41, 1_700_000_000);
    let report = public_certified_delta_report(&request);
    let mut state = NnsRegistryReplayState::new();

    let error = apply_nns_certified_registry_delta_batch(
        &mut state,
        &request,
        &report,
        NnsRegistryReplayLimits::new(100, 1_024 * 1_024),
    )
    .expect_err("state must begin at the batch request version");

    assert!(matches!(
        error,
        NnsRegistryReplayError::VersionMismatch {
            state_version: 0,
            requested_version: 41,
        }
    ));
    assert_eq!(state.through_version(), 0);
    assert!(state.is_empty());

    let mut session = NnsRegistryReplaySession::new(NnsRegistryReplaySessionLimits::new(
        100,
        1,
        1,
        1_024,
        NnsRegistryReplayLimits::new(100, 1_024 * 1_024),
    ));
    let error = session
        .apply_batch(&request, &report)
        .expect_err("session must begin at Registry version zero");
    assert!(matches!(
        error,
        NnsRegistryReplayError::VersionMismatch {
            state_version: 0,
            requested_version: 41,
        }
    ));
    assert_eq!(session.selected_version(), None);
    assert!(session.state().is_empty());
}

fn public_certified_delta_report(
    request: &NnsCertifiedRegistryDeltaBatchRequest,
) -> NnsCertifiedRegistryDeltaBatchReport {
    NnsCertifiedRegistryDeltaBatchReport {
        schema_version: 2,
        network: "ic".to_string(),
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        requested_version: request.requested_version,
        certified_latest_version: 42,
        first_version: Some(42),
        last_version: Some(42),
        version_count: 1,
        mutation_count: 1,
        precondition_count: 0,
        inline_value_bytes: 1,
        chunk_value_bytes: 0,
        value_bytes: 1,
        chunk_reference_count: 0,
        more_available: false,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint.clone(),
        fetched_by: "ic-query".to_string(),
        query_call_count: 1,
        chunk_query_call_count: 0,
        certified_response_bytes: 64,
        chunk_response_bytes: 0,
        response_bytes: 64,
        limits: public_certified_delta_limits(),
        versions: vec![NnsCertifiedRegistryDeltaVersion {
            version: 42,
            timestamp_nanoseconds: 1_700_000_000_000_000_000,
            mutations: vec![NnsCertifiedRegistryMutation {
                mutation_type: 4,
                mutation_kind: NnsCertifiedRegistryMutationKind::Upsert,
                key_hex: "61".to_string(),
                value_encoding: NnsCertifiedRegistryValueEncoding::Inline,
                chunk_sha256_hexes: Vec::new(),
                value_hex: Some("62".to_string()),
            }],
            preconditions: Vec::new(),
        }],
        certification: public_registry_certification(),
    }
}

const fn public_certified_delta_limits() -> NnsCertifiedRegistryDeltaLimits {
    #[cfg(feature = "nns-host")]
    {
        nns_certified_registry_delta_limits()
    }
    #[cfg(not(feature = "nns-host"))]
    {
        NnsCertifiedRegistryDeltaLimits {
            max_versions: 1_000,
            max_mutations: 65_536,
            max_preconditions: 65_536,
            max_key_bytes: 4_096,
            max_inline_value_bytes: 2 * 1_024 * 1_024,
            max_chunk_references: 64,
            max_chunk_bytes: 1_800_000,
            max_reconstructed_value_bytes: 10 * 1_024 * 1_024,
            max_value_bytes: 16 * 1_024 * 1_024,
            max_chunk_response_bytes: 32 * 1_024 * 1_024,
            max_response_body_bytes: 8 * 1_024 * 1_024,
        }
    }
}

fn public_registry_certification() -> NnsRegistryCertification {
    NnsRegistryCertification {
        certificate_verified: true,
        certificate_time_nanos: 1_700_000_000_000_000_000,
        certificate_time: "2023-11-14T22:13:20Z".to_string(),
        root_key_digest: "ab".repeat(32),
        certificate_hex: "cd".repeat(8),
        certificate_bytes: 8,
        hash_tree_hex: "ef".repeat(4),
        hash_tree_bytes: 4,
    }
}

#[test]
fn public_nns_node_api_is_constructible_and_renderable() {
    let cache = NnsInventoryCacheRequest::new(".", "ic");
    let list_request = NnsNodeListRequest::new(cache.clone(), "https://icp-api.io", 1_700_000_000)
        .with_subnet("tdb26-jop6g")
        .with_subnet_kind(SubnetKind::Application)
        .with_data_center("zh1");

    assert_eq!(
        list_request.filters.subnet_kind,
        Some(SubnetKind::Application)
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
    assert!(list_text.contains(SubnetKind::Application.as_str()));
    assert!(verbose_text.contains("source_endpoint: https://icp-api.io"));
    assert!(verbose_text.contains("tdb26-jop6g-7sc54-foywl"));

    let info_request = NnsInventoryInfoRequest::new(
        cache,
        "https://icp-api.io",
        node.node_principal.clone(),
        1_700_000_000,
    );
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
    let cache = NnsInventoryCacheRequest::new(".", "ic");
    let request = NnsInventoryListRequest::new(cache.clone(), "https://icp-api.io", 1_700_000_000);
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

    let info_request = NnsInventoryInfoRequest::new(
        cache,
        "https://icp-api.io",
        data_center.data_center_id.clone(),
        1_700_000_000,
    );
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
    let cache = NnsInventoryCacheRequest::new(".", "ic");
    let request = NnsInventoryListRequest::new(cache.clone(), "https://icp-api.io", 1_700_000_000);
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

    let info_request = NnsInventoryInfoRequest::new(
        cache,
        "https://icp-api.io",
        provider.node_provider_principal.clone(),
        1_700_000_000,
    );
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
    let cache = NnsInventoryCacheRequest::new(".", "ic");
    let request = NnsInventoryListRequest::new(cache.clone(), "https://icp-api.io", 1_700_000_000);
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

    let info_request = NnsInventoryInfoRequest::new(
        cache,
        "https://icp-api.io",
        operator.node_operator_principal.clone(),
        1_700_000_000,
    );
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

#[cfg(feature = "nns-host")]
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

#[cfg(feature = "nns-host")]
#[test]
fn public_nns_inventory_host_api_accepts_custom_source_adapters() {
    let root = temp_root("nns-inventory-source-public-api");

    assert_public_nns_node_custom_source_api(&root);
    assert_public_nns_data_center_custom_source_api(&root);
    assert_public_nns_node_provider_custom_source_api(&root);
    assert_public_nns_node_operator_custom_source_api(&root);

    let _ = fs::remove_dir_all(root);
}

#[cfg(feature = "nns-host")]
fn assert_public_nns_node_custom_source_api(root: &Path) {
    let node_cache = NnsInventoryCacheRequest::new(root.join("node"), "ic");
    let node_list_request = NnsNodeListRequest::new(
        node_cache.clone(),
        DEFAULT_NNS_NODE_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    let node_info_request = NnsInventoryInfoRequest::new(
        node_cache.clone(),
        DEFAULT_NNS_NODE_SOURCE_ENDPOINT,
        sample_nns_node_row().node_principal,
        1_700_000_000,
    );
    let node_refresh_request = NnsInventoryRefreshRequest::new(
        node_cache,
        DEFAULT_NNS_NODE_SOURCE_ENDPOINT,
        1_700_000_000,
        DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS,
    )
    .with_dry_run(true);
    let node_list =
        build_nns_node_list_report_with_source(&node_list_request, &FixtureNnsNodeSource)
            .expect("node list report");
    let node_info =
        build_nns_node_info_report_with_source(&node_info_request, &FixtureNnsNodeSource)
            .expect("node info report");
    let node_refresh =
        refresh_nns_node_report_with_source(&node_refresh_request, &FixtureNnsNodeSource)
            .expect("node refresh report");

    assert_eq!(node_list.node_count, 1);
    assert_eq!(
        node_info.node_principal,
        sample_nns_node_row().node_principal
    );
    assert_eq!(node_refresh.node_count, 1);
}

#[cfg(feature = "nns-host")]
fn assert_public_nns_data_center_custom_source_api(root: &Path) {
    let data_center_cache = NnsInventoryCacheRequest::new(root.join("data-center"), "ic");
    let data_center_list_request = NnsInventoryListRequest::new(
        data_center_cache.clone(),
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    let data_center_info_request = NnsInventoryInfoRequest::new(
        data_center_cache.clone(),
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
        sample_nns_data_center_row().data_center_id,
        1_700_000_000,
    );
    let data_center_refresh_request = NnsInventoryRefreshRequest::new(
        data_center_cache,
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
        1_700_000_000,
        DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS,
    )
    .with_dry_run(true);
    let data_center_list = build_nns_data_center_list_report_with_source(
        &data_center_list_request,
        &FixtureNnsDataCenterSource,
    )
    .expect("data-center list report");
    let data_center_info = build_nns_data_center_info_report_with_source(
        &data_center_info_request,
        &FixtureNnsDataCenterSource,
    )
    .expect("data-center info report");
    let data_center_refresh = refresh_nns_data_center_report_with_source(
        &data_center_refresh_request,
        &FixtureNnsDataCenterSource,
    )
    .expect("data-center refresh report");

    assert_eq!(data_center_list.data_center_count, 1);
    assert_eq!(
        data_center_info.data_center_id,
        sample_nns_data_center_row().data_center_id
    );
    assert_eq!(data_center_refresh.data_center_count, 1);
}

#[cfg(feature = "nns-host")]
fn assert_public_nns_node_provider_custom_source_api(root: &Path) {
    let node_provider_cache = NnsInventoryCacheRequest::new(root.join("node-provider"), "ic");
    let node_provider_list_request = NnsInventoryListRequest::new(
        node_provider_cache.clone(),
        DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    let node_provider_info_request = NnsInventoryInfoRequest::new(
        node_provider_cache.clone(),
        DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
        sample_nns_node_provider_row().node_provider_principal,
        1_700_000_000,
    );
    let node_provider_refresh_request = NnsInventoryRefreshRequest::new(
        node_provider_cache,
        DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
        1_700_000_000,
        DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS,
    )
    .with_dry_run(true);
    let node_provider_list = build_nns_node_provider_list_report_with_source(
        &node_provider_list_request,
        &FixtureNnsNodeProviderSource,
    )
    .expect("node-provider list report");
    let node_provider_info = build_nns_node_provider_info_report_with_source(
        &node_provider_info_request,
        &FixtureNnsNodeProviderSource,
    )
    .expect("node-provider info report");
    let node_provider_refresh = refresh_nns_node_provider_report_with_source(
        &node_provider_refresh_request,
        &FixtureNnsNodeProviderSource,
    )
    .expect("node-provider refresh report");

    assert_eq!(node_provider_list.node_provider_count, 1);
    assert_eq!(
        node_provider_info.node_provider_principal,
        sample_nns_node_provider_row().node_provider_principal
    );
    assert_eq!(node_provider_refresh.node_provider_count, 1);
}

#[cfg(feature = "nns-host")]
fn assert_public_nns_node_operator_custom_source_api(root: &Path) {
    let node_operator_cache = NnsInventoryCacheRequest::new(root.join("node-operator"), "ic");
    let node_operator_list_request = NnsInventoryListRequest::new(
        node_operator_cache.clone(),
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    let node_operator_info_request = NnsInventoryInfoRequest::new(
        node_operator_cache.clone(),
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        sample_nns_node_operator_row().node_operator_principal,
        1_700_000_000,
    );
    let node_operator_refresh_request = NnsInventoryRefreshRequest::new(
        node_operator_cache,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        1_700_000_000,
        DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS,
    )
    .with_dry_run(true);
    let node_operator_list = build_nns_node_operator_list_report_with_source(
        &node_operator_list_request,
        &FixtureNnsNodeOperatorSource,
    )
    .expect("node-operator list report");
    let node_operator_info = build_nns_node_operator_info_report_with_source(
        &node_operator_info_request,
        &FixtureNnsNodeOperatorSource,
    )
    .expect("node-operator info report");
    let node_operator_refresh = refresh_nns_node_operator_report_with_source(
        &node_operator_refresh_request,
        &FixtureNnsNodeOperatorSource,
    )
    .expect("node-operator refresh report");

    assert_eq!(node_operator_list.node_operator_count, 1);
    assert_eq!(
        node_operator_info.node_operator_principal,
        sample_nns_node_operator_row().node_operator_principal
    );
    assert_eq!(node_operator_refresh.node_operator_count, 1);
}

#[cfg(feature = "nns-host")]
struct FixtureNnsNodeSource;

#[cfg(feature = "nns-host")]
impl NnsNodeSource for FixtureNnsNodeSource {
    fn fetch_node_list_report(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsNodeListReport, NnsNodeHostError> {
        assert_inventory_source_request(
            &request.network,
            &request.endpoint,
            &request.fetched_at,
            &request.fetched_by,
        );
        let mut report = sample_nns_node_list_report();
        report.network.clone_from(&request.network);
        report.fetched_at.clone_from(&request.fetched_at);
        report.source_endpoint.clone_from(&request.endpoint);
        report.fetched_by.clone_from(&request.fetched_by);
        Ok(report)
    }
}

#[cfg(feature = "nns-host")]
struct FixtureNnsDataCenterSource;

#[cfg(feature = "nns-host")]
impl NnsDataCenterSource for FixtureNnsDataCenterSource {
    fn fetch_data_center_list_report(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsDataCenterListReport, NnsDataCenterHostError> {
        assert_inventory_source_request(
            &request.network,
            &request.endpoint,
            &request.fetched_at,
            &request.fetched_by,
        );
        let mut report = sample_nns_data_center_list_report();
        report.network.clone_from(&request.network);
        report.fetched_at.clone_from(&request.fetched_at);
        report.source_endpoint.clone_from(&request.endpoint);
        report.fetched_by.clone_from(&request.fetched_by);
        Ok(report)
    }
}

#[cfg(feature = "nns-host")]
struct FixtureNnsNodeProviderSource;

#[cfg(feature = "nns-host")]
impl NnsNodeProviderSource for FixtureNnsNodeProviderSource {
    fn fetch_node_provider_list_report(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsNodeProviderListReport, NnsNodeProviderHostError> {
        assert_inventory_source_request(
            &request.network,
            &request.endpoint,
            &request.fetched_at,
            &request.fetched_by,
        );
        let mut report = sample_nns_node_provider_list_report();
        report.network.clone_from(&request.network);
        report.fetched_at.clone_from(&request.fetched_at);
        report.source_endpoint.clone_from(&request.endpoint);
        report.fetched_by.clone_from(&request.fetched_by);
        Ok(report)
    }
}

#[cfg(feature = "nns-host")]
struct FixtureNnsNodeOperatorSource;

#[cfg(feature = "nns-host")]
impl NnsNodeOperatorSource for FixtureNnsNodeOperatorSource {
    fn fetch_node_operator_list_report(
        &self,
        request: &NnsSourceRequest,
    ) -> Result<NnsNodeOperatorListReport, NnsNodeOperatorHostError> {
        assert_inventory_source_request(
            &request.network,
            &request.endpoint,
            &request.fetched_at,
            &request.fetched_by,
        );
        let mut report = sample_nns_node_operator_list_report();
        report.network.clone_from(&request.network);
        report.fetched_at.clone_from(&request.fetched_at);
        report.source_endpoint.clone_from(&request.endpoint);
        report.fetched_by.clone_from(&request.fetched_by);
        Ok(report)
    }
}

#[cfg(feature = "nns-host")]
fn assert_inventory_source_request(
    network: &str,
    endpoint: &str,
    fetched_at: &str,
    fetched_by: &str,
) {
    assert_eq!(network, "ic");
    assert_eq!(endpoint, "https://icp-api.io");
    assert!(!fetched_at.is_empty());
    assert_eq!(fetched_by, "ic-query");
}

#[test]
fn public_nns_proposal_api_is_constructible_and_renderable() {
    let request = NnsProposalListRequest::new("ic", "https://icp-api.io", 1_700_000_000, 25)
        .with_before_proposal_id(132_500)
        .with_status(NnsProposalStatusFilter::Executed)
        .with_reward_status(NnsProposalRewardStatusFilter::Settled)
        .with_topic(NnsProposalTopicFilter::Governance)
        .with_proposer_neuron_id(12_345)
        .with_query("subnet")
        .with_sort(NnsProposalListSort::TallyTime)
        .with_sort_direction(NnsProposalSortDirection::Desc)
        .with_verbose(true);

    assert_eq!(request.status.as_str(), "executed");
    assert_eq!(request.reward_status.as_str(), "settled");
    assert_eq!(request.topic.as_str(), "governance");
    assert_eq!(request.sort.direction_label(request.sort_direction), "desc");

    let proposal = sample_nns_proposal_row();
    let list_report = NnsProposalListReport {
        schema_version: 1,
        network: request.network,
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        data_source: ReportDataSource::Cache,
        cache_path: Some("/cache/nns/ic/governance/proposals/full.json".to_string()),
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
        result_scope: ReportResultScope::CompleteCache,
        verbose: request.verbose,
        proposal_count: 1,
        proposals: vec![proposal.clone()],
    };

    let list_json = serde_json::to_value(&list_report).expect("serialize NNS proposal list report");
    let list_text = nns_proposal_list_report_text(&list_report);

    assert_eq!(list_json["proposals"][0]["topic_text"], "governance");
    assert_eq!(list_json["proposals"][0]["status_text"], "executed");
    assert_eq!(list_json["proposals"][0]["reward_status_text"], "settled");
    assert_eq!(list_json["proposals"][0]["ballots"][0]["vote_text"], "yes");
    assert!(list_text.contains("proposal_count: 1"));
    assert!(list_text.contains("topic_filter: governance"));
    assert!(list_text.contains("proposal_details:"));
    assert!(list_text.contains("title: Upgrade subnet"));

    let detail_request =
        NnsProposalRequest::new("ic", "https://icp-api.io", 1_700_000_000, 132_411)
            .with_show_ballots(true);
    let detail_report = NnsProposalReport {
        schema_version: 1,
        network: detail_request.network,
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: detail_request.source_endpoint,
        fetched_by: "ic-query".to_string(),
        data_source: ReportDataSource::Live,
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
    let request = NnsTopologyReadRequest::new(".", "ic", "https://icp-api.io", 1_700_000_000);
    assert_eq!(request.network, "ic");

    let registry_version = sample_topology_registry_version_row();
    let summary = NnsTopologySummaryReport {
        schema_version: 1,
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
fn public_nns_topology_coverage_and_check_api_is_constructible_and_renderable() {
    let check = NnsTopologyCheckReport {
        schema_version: 1,
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        status: NnsTopologyAssessmentStatus::Ok,
        registry_source_count: 1,
        registry_version_min: Some(42),
        registry_version_max: Some(42),
        registry_versions_aligned: true,
        stale_source_count: 0,
        unknown_freshness_source_count: 0,
        subnet_catalog_stale: false,
        subnet_catalog_stale_reason: "fresh".to_string(),
        known_join_count: 11,
        unknown_join_count: 0,
        join_coverage: "100.0%".to_string(),
        checks: vec![NnsTopologyCheckRow {
            check: "registry_versions".to_string(),
            status: NnsTopologyAssessmentStatus::Ok,
            detail: "1 source at registry version 42".to_string(),
        }],
    };
    assert!(nns_topology_check_report_text(&check).contains("registry_versions"));

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
        status: NnsTopologyAssessmentStatus::Attention,
        gap_count: 1,
        gaps: vec![NnsTopologyGapRow {
            subject_kind: NnsTopologyGapSubjectKind::Node,
            subject: "zh3jp-xqaaa-aaaar-qaada-cai".to_string(),
            missing_relation: NnsTopologyGapRelationKind::NodeOperator,
            referenced_id: "qoctq-giaaa-aaaar-qaada-cai".to_string(),
        }],
    };
    assert!(nns_topology_gaps_report_text(&gaps).contains("node_operator"));

    let capacity = NnsTopologyCapacityReport {
        schema_version: 1,
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        status: NnsTopologyAssessmentStatus::Attention,
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
            status: NnsTopologyCapacityStatus::Over,
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
        registry_versions: vec![NnsTopologyRegistryVersionRow {
            source: "nodes".to_string(),
            registry_version: 42,
            fetched_at: "2023-11-14T22:13:20Z".to_string(),
            source_endpoint: "https://icp-api.io".to_string(),
            stale: None,
        }],
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
            status: NnsTopologyProviderStatus::Over,
        }],
    };
    assert!(nns_topology_providers_report_text(&providers).contains("over"));

    let refresh_request =
        NnsTopologyRefreshRequest::new(".", "ic", "https://icp-api.io", 1_700_000_000, 1_800)
            .with_dry_run(true);
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
            cache_path: "/cache/nns/ic/subnet-catalog/catalog.json".to_string(),
            refresh_lock_path: "/cache/nns/ic/subnet-catalog/refresh.lock".to_string(),
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

#[cfg(feature = "nns-host")]
#[test]
fn public_nns_topology_host_api_accepts_custom_source_adapter() {
    let source = FixtureNnsTopologySource;
    let request = NnsTopologyReadRequest::new(
        ".",
        "ic",
        DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT,
        1_700_000_000,
    );

    let summary = build_nns_topology_summary_report_with_source(&request, &source)
        .expect("topology summary report");
    let coverage = build_nns_topology_coverage_report_with_source(
        &NnsTopologyReadRequest::new(
            ".",
            "ic",
            DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT,
            1_700_000_000,
        ),
        &source,
    )
    .expect("topology coverage report");
    let check = build_nns_topology_check_report_with_source(
        &NnsTopologyReadRequest::new(
            ".",
            "ic",
            DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT,
            1_700_000_000,
        ),
        &source,
    )
    .expect("topology check report");
    let versions = topology_versions_report_with_source(&source);
    let refresh = topology_refresh_report_with_source(&source);

    assert_eq!(summary.network, "ic");
    assert_eq!(summary.subnet_count, 1);
    assert_eq!(coverage.node_count, 1);
    assert_eq!(check.network, "ic");
    assert_eq!(versions.source_count, 5);
    assert_eq!(refresh.component_count, 5);
    assert!(refresh.dry_run);
    assert_topology_direct_reports_with_source(&source);
}

#[cfg(feature = "nns-host")]
fn topology_versions_report_with_source(
    source: &dyn NnsTopologySource,
) -> NnsTopologyVersionsReport {
    build_nns_topology_versions_report_with_source(
        &NnsTopologyReadRequest::new(
            ".",
            "ic",
            DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT,
            1_700_000_000,
        ),
        source,
    )
    .expect("topology versions report")
}

#[cfg(feature = "nns-host")]
fn topology_refresh_report_with_source(
    source: &dyn NnsTopologyRefreshSource,
) -> NnsTopologyRefreshReport {
    refresh_nns_topology_report_with_source(
        &NnsTopologyRefreshRequest::new(
            ".",
            "ic",
            DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT,
            1_700_000_000,
            1_800,
        )
        .with_dry_run(true),
        source,
    )
    .expect("topology refresh report")
}

#[cfg(feature = "nns-host")]
fn assert_topology_direct_reports_with_source(source: &dyn NnsTopologySource) {
    let gaps_request: NnsTopologyReadRequest = topology_read_request();
    let capacity_request: NnsTopologyReadRequest = topology_read_request();
    let regions_request: NnsTopologyReadRequest = topology_read_request();
    let providers_request: NnsTopologyReadRequest = topology_read_request();
    let gaps =
        build_nns_topology_gaps_report_with_source(&gaps_request, source).expect("gaps report");
    let capacity = build_nns_topology_capacity_report_with_source(&capacity_request, source)
        .expect("capacity report");
    let regions = build_nns_topology_regions_report_with_source(&regions_request, source)
        .expect("regions report");
    let providers = build_nns_topology_providers_report_with_source(&providers_request, source)
        .expect("providers report");

    assert_eq!(gaps.network, "ic");
    assert_eq!(capacity.node_operator_count, 1);
    assert_eq!(regions.region_count, 1);
    assert_eq!(providers.registered_node_provider_count, 1);
}

#[cfg(feature = "nns-host")]
fn topology_read_request() -> NnsTopologyReadRequest {
    NnsTopologyReadRequest::new(
        ".",
        "ic",
        DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT,
        1_700_000_000,
    )
}

#[cfg(feature = "nns-host")]
struct FixtureNnsTopologySource;

#[cfg(feature = "nns-host")]
impl NnsTopologySource for FixtureNnsTopologySource {
    fn fetch_subnet_catalog_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<SubnetCatalogListReport, NnsTopologyHostError> {
        assert_topology_source_request(request);
        let mut report = sample_subnet_catalog_list_report();
        report.network.clone_from(&request.network);
        report.fetched_at.clone_from(&request.fetched_at);
        Ok(report)
    }

    fn fetch_node_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsNodeListReport, NnsTopologyHostError> {
        assert_topology_source_request(request);
        let mut report = sample_nns_node_list_report();
        stamp_topology_component_report(request, &mut report.network, &mut report.source_endpoint);
        report.fetched_at.clone_from(&request.fetched_at);
        report.fetched_by.clone_from(&request.fetched_by);
        Ok(report)
    }

    fn fetch_node_provider_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsNodeProviderListReport, NnsTopologyHostError> {
        assert_topology_source_request(request);
        let mut report = sample_nns_node_provider_list_report();
        stamp_topology_component_report(request, &mut report.network, &mut report.source_endpoint);
        report.fetched_at.clone_from(&request.fetched_at);
        report.fetched_by.clone_from(&request.fetched_by);
        Ok(report)
    }

    fn fetch_node_operator_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsNodeOperatorListReport, NnsTopologyHostError> {
        assert_topology_source_request(request);
        let mut report = sample_nns_node_operator_list_report();
        stamp_topology_component_report(request, &mut report.network, &mut report.source_endpoint);
        report.fetched_at.clone_from(&request.fetched_at);
        report.fetched_by.clone_from(&request.fetched_by);
        Ok(report)
    }

    fn fetch_data_center_list_report(
        &self,
        request: &NnsTopologySourceRequest,
    ) -> Result<NnsDataCenterListReport, NnsTopologyHostError> {
        assert_topology_source_request(request);
        let mut report = sample_nns_data_center_list_report();
        stamp_topology_component_report(request, &mut report.network, &mut report.source_endpoint);
        report.fetched_at.clone_from(&request.fetched_at);
        report.fetched_by.clone_from(&request.fetched_by);
        Ok(report)
    }
}

#[cfg(feature = "nns-host")]
impl NnsTopologyRefreshSource for FixtureNnsTopologySource {
    fn refresh_subnet_catalog_report(
        &self,
        request: &NnsTopologyRefreshSourceRequest,
    ) -> Result<SubnetCatalogRefreshReport, NnsTopologyHostError> {
        assert_topology_refresh_source_request(request);
        Ok(sample_subnet_catalog_refresh_report())
    }

    fn refresh_node_report(
        &self,
        request: &NnsTopologyRefreshSourceRequest,
    ) -> Result<NnsNodeRefreshReport, NnsTopologyHostError> {
        assert_topology_refresh_source_request(request);
        Ok(sample_nns_node_refresh_report(&request.cache_root))
    }

    fn refresh_node_provider_report(
        &self,
        request: &NnsTopologyRefreshSourceRequest,
    ) -> Result<NnsNodeProviderRefreshReport, NnsTopologyHostError> {
        assert_topology_refresh_source_request(request);
        Ok(sample_nns_node_provider_refresh_report(&request.cache_root))
    }

    fn refresh_node_operator_report(
        &self,
        request: &NnsTopologyRefreshSourceRequest,
    ) -> Result<NnsNodeOperatorRefreshReport, NnsTopologyHostError> {
        assert_topology_refresh_source_request(request);
        Ok(sample_nns_node_operator_refresh_report(&request.cache_root))
    }

    fn refresh_data_center_report(
        &self,
        request: &NnsTopologyRefreshSourceRequest,
    ) -> Result<NnsDataCenterRefreshReport, NnsTopologyHostError> {
        assert_topology_refresh_source_request(request);
        Ok(sample_nns_data_center_refresh_report(&request.cache_root))
    }
}

#[cfg(feature = "nns-host")]
fn stamp_topology_component_report(
    request: &NnsTopologySourceRequest,
    network: &mut String,
    source_endpoint: &mut String,
) {
    network.clone_from(&request.network);
    source_endpoint.clone_from(&request.endpoint);
}

#[cfg(feature = "nns-host")]
fn assert_topology_source_request(request: &NnsTopologySourceRequest) {
    assert_eq!(request.cache_root, PathBuf::from("."));
    assert_eq!(request.network, "ic");
    assert_eq!(request.endpoint, DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT);
    assert_eq!(request.now_unix_secs, 1_700_000_000);
    assert!(!request.fetched_at.is_empty());
    assert_eq!(request.fetched_by, "ic-query");
}

#[cfg(feature = "nns-host")]
fn assert_topology_refresh_source_request(request: &NnsTopologyRefreshSourceRequest) {
    assert_eq!(request.cache_root, PathBuf::from("."));
    assert_eq!(request.network, "ic");
    assert_eq!(request.endpoint, DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT);
    assert_eq!(request.now_unix_secs, 1_700_000_000);
    assert_eq!(request.lock_stale_after_seconds, 1_800);
    assert!(request.dry_run);
    assert!(!request.fetched_at.is_empty());
    assert_eq!(request.fetched_by, "ic-query");
}

#[cfg(feature = "nns-host")]
type RefreshFn<Request, Report, Error> = fn(&Request) -> Result<Report, Error>;

#[cfg(feature = "nns-host")]
type RequestReportFn<Request, Report, Error> = fn(&Request) -> Result<Report, Error>;

#[cfg(feature = "nns-host")]
#[test]
fn public_nns_proposal_host_api_reads_complete_cache_without_cli() {
    let root = temp_root("nns-proposal-host-public-api");
    write_nns_proposal_fixture_cache(&root);

    let cache_list_request = NnsGovernanceCacheRequest::new(&root, "ic");
    let cache_status_request = NnsGovernanceCacheRequest::new(&root, "ic");
    let list_request = NnsProposalListRequest::new(
        "ic",
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT,
        1_700_000_000,
        25,
    )
    .with_status(NnsProposalStatusFilter::Executed)
    .with_reward_status(NnsProposalRewardStatusFilter::Settled)
    .with_topic(NnsProposalTopicFilter::Governance)
    .with_proposer_neuron_id(12_345)
    .with_query("subnet")
    .with_sort(NnsProposalListSort::TallyTime)
    .with_sort_direction(NnsProposalSortDirection::Desc);
    let detail_request = NnsProposalRequest::new(
        "ic",
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT,
        1_700_000_000,
        132_411,
    )
    .with_show_ballots(true);
    let refresh_request = NnsGovernanceRefreshRequest::new(
        &root,
        "ic",
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT,
        1_700_000_000,
        100,
    )
    .with_max_pages(Some(1));

    let cache_list =
        build_nns_proposal_cache_list_report(&cache_list_request).expect("cache list report");
    let cache_status =
        build_nns_proposal_cache_status_report(&cache_status_request).expect("cache status report");
    let proposal_list = build_nns_proposal_list_report_from_cache(&list_request, &root)
        .expect("cached list lookup")
        .expect("cached list report");
    let cached_detail = build_nns_proposal_report_from_cache(&detail_request, &root)
        .expect("cached detail lookup")
        .expect("cached detail report");
    let refresh_report = sample_nns_proposal_refresh_report(&root);

    assert_eq!(cache_list_request.cache_root(), root.as_path());
    assert_eq!(cache_status_request.cache_root(), root.as_path());
    assert_eq!(cache_list.cache_count, 1);
    assert!(cache_status.found);
    assert_eq!(proposal_list.data_source.as_str(), "cache");
    assert_eq!(proposal_list.proposal_count, 1);
    assert_eq!(cached_detail.data_source.as_str(), "cache");
    assert_eq!(cached_detail.proposal_id, 132_411);
    assert!(nns_proposal_cache_path(&root, "ic").is_file());
    assert!(nns_proposal_cache_root(&root, "ic").ends_with("proposals"));
    assert!(nns_proposal_refresh_lock_path(&root, "ic").ends_with("full.refresh.lock"));
    assert!(nns_proposal_refresh_attempt_path(&root, "ic").ends_with("full.refresh-attempt.json"));
    assert_eq!(DEFAULT_NNS_PROPOSAL_REFRESH_LOCK_STALE_SECONDS, 1_800);
    assert!(nns_proposal_cache_list_report_text(&cache_list).contains("cache_count: 1"));
    assert!(nns_proposal_cache_status_report_text(&cache_status).contains("found: yes"));
    assert!(nns_proposal_refresh_report_text(&refresh_report).contains("proposal_count: 1"));
    assert!(request_report_api_accepts_public_types(
        build_nns_proposal_list_report,
        &list_request
    ));
    assert!(request_report_api_accepts_public_types(
        build_nns_proposal_report,
        &detail_request
    ));
    assert!(request_report_api_accepts_public_types(
        refresh_nns_proposal_cache,
        &refresh_request
    ));

    let _ = fs::remove_dir_all(root);
}

#[cfg(feature = "nns-host")]
#[test]
fn public_nns_proposal_host_api_accepts_custom_source_adapter() {
    let root = temp_root("nns-proposal-source-public-api");
    let source = FixtureNnsProposalSource;
    let list_request = NnsProposalListRequest::new(
        "ic",
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT,
        1_700_000_000,
        25,
    )
    .with_status(NnsProposalStatusFilter::Executed)
    .with_reward_status(NnsProposalRewardStatusFilter::Settled)
    .with_topic(NnsProposalTopicFilter::Governance);
    let detail_request = NnsProposalRequest::new(
        "ic",
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT,
        1_700_000_000,
        132_411,
    );
    let refresh_request = NnsGovernanceRefreshRequest::new(
        &root,
        "ic",
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT,
        1_700_000_000,
        2,
    );

    let list = build_nns_proposal_list_report_with_source(&list_request, &source)
        .expect("proposal list report");
    let detail = build_nns_proposal_report_with_source(&detail_request, &source)
        .expect("proposal detail report");
    let refresh = refresh_nns_proposal_cache_with_source(&refresh_request, &source)
        .expect("proposal refresh report");

    assert_eq!(list.proposal_count, 1);
    assert_eq!(list.data_source.as_str(), "live");
    assert_eq!(detail.proposal_id, 132_411);
    assert_eq!(detail.proposal.title.as_deref(), Some("Upgrade subnet"));
    assert_eq!(refresh.proposal_count, 1);
    assert!(refresh.complete);
    assert!(nns_proposal_cache_path(&root, "ic").is_file());

    let _ = fs::remove_dir_all(root);
}

#[cfg(feature = "nns-host")]
struct FixtureNnsProposalSource;

#[cfg(feature = "nns-host")]
impl NnsProposalSource for FixtureNnsProposalSource {
    fn fetch_proposals(
        &self,
        request: &NnsSourceRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        status: NnsProposalStatusFilter,
        reward_status: NnsProposalRewardStatusFilter,
    ) -> Result<Vec<NnsProposalRow>, NnsProposalHostError> {
        assert_proposal_source_request(request);
        match (limit, before_proposal_id, status, reward_status) {
            (
                25,
                None,
                NnsProposalStatusFilter::Executed,
                NnsProposalRewardStatusFilter::Settled,
            )
            | (2, None, NnsProposalStatusFilter::Any, NnsProposalRewardStatusFilter::Any) => {
                Ok(vec![sample_nns_proposal_row()])
            }
            other => panic!("unexpected proposal source call: {other:?}"),
        }
    }

    fn fetch_proposal(
        &self,
        request: &NnsSourceRequest,
        proposal_id: u64,
    ) -> Result<NnsProposalRow, NnsProposalHostError> {
        assert_proposal_source_request(request);
        assert_eq!(proposal_id, 132_411);
        Ok(sample_nns_proposal_row())
    }
}

#[cfg(feature = "nns-host")]
fn assert_proposal_source_request(request: &NnsSourceRequest) {
    assert_eq!(request.network, "ic");
    assert_eq!(request.endpoint, DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT);
    assert!(!request.fetched_at.is_empty());
    assert_eq!(request.fetched_by, "ic-query");
}

#[cfg(feature = "nns-host")]
fn write_nns_inventory_fixture_caches(root: &Path) {
    write_json_cache(
        root,
        &nns_node_cache_path(root, "ic"),
        &sample_nns_node_list_report(),
    );
    write_json_cache(
        root,
        &nns_data_center_cache_path(root, "ic"),
        &sample_nns_data_center_list_report(),
    );
    write_json_cache(
        root,
        &nns_node_provider_cache_path(root, "ic"),
        &sample_nns_node_provider_list_report(),
    );
    write_json_cache(
        root,
        &nns_node_operator_cache_path(root, "ic"),
        &sample_nns_node_operator_list_report(),
    );
}

#[cfg(feature = "nns-host")]
fn write_nns_proposal_fixture_cache(root: &Path) {
    write_json_cache(
        root,
        &nns_proposal_cache_path(root, "ic"),
        &serde_json::json!({
            "schema_version": 1,
            "network": "ic",
            "source_endpoint": DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT,
            "fetched_at": "2023-11-14T22:13:20Z",
            "fetched_by": "ic-query",
            "domain": "nns",
            "entity": "governance",
            "collection": "proposals",
            "scope": "full",
            "governance_canister_id": "rrkah-fqaaa-aaaaa-aaaaq-cai",
            "completeness": {
                "status": "api_exhausted",
                "page_size": 100,
                "page_count": 1,
                "row_count": 1,
                "point_in_time_guaranteed": false
            },
            "proposals": [sample_nns_proposal_row()]
        }),
    );
}

#[cfg(feature = "nns-host")]
fn assert_public_nns_node_host_api(root: &Path) {
    let cache = NnsInventoryCacheRequest::new(root, "ic");
    let request = NnsNodeListRequest::new(
        cache.clone(),
        DEFAULT_NNS_NODE_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    let list = build_nns_node_list_report(&request).expect("build node list from cache");
    let info = build_nns_node_info_report(&NnsInventoryInfoRequest::new(
        cache.clone(),
        DEFAULT_NNS_NODE_SOURCE_ENDPOINT,
        sample_nns_node_row().node_principal,
        1_700_000_000,
    ))
    .expect("build node info from cache");
    let refresh = sample_nns_node_refresh_report(root);
    let refresh_request = NnsInventoryRefreshRequest::new(
        cache,
        DEFAULT_NNS_NODE_SOURCE_ENDPOINT,
        1_700_000_000,
        DEFAULT_NODE_REFRESH_LOCK_STALE_SECONDS,
    )
    .with_dry_run(true);

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

#[cfg(feature = "nns-host")]
fn assert_public_nns_data_center_host_api(root: &Path) {
    let cache = NnsInventoryCacheRequest::new(root, "ic");
    let request = NnsInventoryListRequest::new(
        cache.clone(),
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    let list =
        build_nns_data_center_list_report(&request).expect("build data-center list from cache");
    let info = build_nns_data_center_info_report(&NnsInventoryInfoRequest::new(
        cache.clone(),
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
        sample_nns_data_center_row().data_center_id,
        1_700_000_000,
    ))
    .expect("build data-center info from cache");
    let refresh = sample_nns_data_center_refresh_report(root);
    let refresh_request = NnsInventoryRefreshRequest::new(
        cache,
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
        1_700_000_000,
        DEFAULT_DATA_CENTER_REFRESH_LOCK_STALE_SECONDS,
    )
    .with_dry_run(true);

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

#[cfg(feature = "nns-host")]
fn assert_public_nns_node_provider_host_api(root: &Path) {
    let cache = NnsInventoryCacheRequest::new(root, "ic");
    let request = NnsInventoryListRequest::new(
        cache.clone(),
        DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    let list =
        build_nns_node_provider_list_report(&request).expect("build node-provider list from cache");
    let info = build_nns_node_provider_info_report(&NnsInventoryInfoRequest::new(
        cache.clone(),
        DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
        sample_nns_node_provider_row().node_provider_principal,
        1_700_000_000,
    ))
    .expect("build node-provider info from cache");
    let refresh = sample_nns_node_provider_refresh_report(root);
    let refresh_request = NnsInventoryRefreshRequest::new(
        cache,
        DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
        1_700_000_000,
        DEFAULT_NODE_PROVIDER_REFRESH_LOCK_STALE_SECONDS,
    )
    .with_dry_run(true);

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

#[cfg(feature = "nns-host")]
fn assert_public_nns_node_operator_host_api(root: &Path) {
    let cache = NnsInventoryCacheRequest::new(root, "ic");
    let request = NnsInventoryListRequest::new(
        cache.clone(),
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    let list =
        build_nns_node_operator_list_report(&request).expect("build node-operator list from cache");
    let info = build_nns_node_operator_info_report(&NnsInventoryInfoRequest::new(
        cache.clone(),
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        sample_nns_node_operator_row().node_operator_principal,
        1_700_000_000,
    ))
    .expect("build node-operator info from cache");
    let refresh = sample_nns_node_operator_refresh_report(root);
    let refresh_request = NnsInventoryRefreshRequest::new(
        cache,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        1_700_000_000,
        DEFAULT_NODE_OPERATOR_REFRESH_LOCK_STALE_SECONDS,
    )
    .with_dry_run(true);

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

#[cfg(feature = "nns-host")]
fn write_json_cache<T>(root: &Path, path: &Path, value: &T)
where
    T: Serialize,
{
    let parent = path.parent().expect("cache parent");
    fs::create_dir_all(parent).expect("create cache parent");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut directory = root.to_path_buf();
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
            .expect("secure cache root");
        for component in parent.strip_prefix(root).expect("parent beneath root") {
            directory.push(component);
            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700))
                .expect("secure cache directory");
        }
    }
    fs::write(
        path,
        serde_json::to_string_pretty(value).expect("serialize fixture cache"),
    )
    .expect("write fixture cache");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("secure cache file");
    }
}

#[cfg(feature = "nns-host")]
#[must_use]
fn refresh_api_accepts_public_types<Request, Report, Error>(
    _refresh: RefreshFn<Request, Report, Error>,
    request: &Request,
) -> bool {
    std::mem::size_of_val(request) > 0
}

#[cfg(feature = "nns-host")]
#[must_use]
fn request_report_api_accepts_public_types<Request, Report, Error>(
    _build: RequestReportFn<Request, Report, Error>,
    request: &Request,
) -> bool {
    std::mem::size_of_val(request) > 0
}

#[cfg(feature = "nns-host")]
#[must_use]
fn temp_root(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("ic-query-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    path
}

#[cfg(feature = "nns-host")]
fn sample_subnet_catalog_list_report() -> SubnetCatalogListReport {
    SubnetCatalogListReport {
        schema_version: 1,
        network: "ic".to_string(),
        catalog_path: "/cache/nns/ic/subnet-catalog/catalog.json".to_string(),
        catalog_schema_version: 2,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        assurance: CatalogAssurance::UncertifiedQuery,
        source_endpoints: vec![DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT.to_string()],
        agreement_digest: None,
        registry_query_call_count: 5,
        catalog_digest: "00".repeat(32),
        cache_disposition: CacheDisposition::CacheHit,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        catalog_stale: false,
        stale_reason: "fresh".to_string(),
        resolver_backend: "local-nns-subnet-catalog".to_string(),
        collector_version: "test".to_string(),
        classification_schema_version: 1,
        classification_policy_digest: "00".repeat(32),
        resolver_schema_version: 1,
        subnets: vec![sample_subnet_catalog_row()],
    }
}

#[cfg(feature = "nns-host")]
fn sample_subnet_catalog_refresh_report() -> SubnetCatalogRefreshReport {
    SubnetCatalogRefreshReport {
        schema_version: 2,
        network: "ic".to_string(),
        catalog_path: "/cache/nns/ic/subnet-catalog/catalog.json".to_string(),
        refresh_lock_path: "/cache/nns/ic/subnet-catalog/refresh.lock".to_string(),
        output_path: None,
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        assurance: CatalogAssurance::UncertifiedQuery,
        source_endpoints: vec![DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT.to_string()],
        agreement_digest: None,
        registry_query_call_count: 5,
        catalog_digest: "00".repeat(32),
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        fetched_by: "ic-query".to_string(),
        collector_version: "test".to_string(),
        classification_schema_version: 1,
        classification_policy_digest: "00".repeat(32),
        resolver_schema_version: 1,
        resolver_backend: "local-nns-subnet-catalog".to_string(),
        dry_run: true,
        wrote_catalog: false,
        replaced_existing_catalog: true,
        subnet_count: 1,
        routing_range_count: 1,
    }
}

#[cfg(feature = "nns-host")]
fn sample_subnet_catalog_row() -> SubnetCatalogSubnetRow {
    let subnet_kind = SubnetKind::Application;
    SubnetCatalogSubnetRow {
        subnet_principal: "tdb26-jop6g-7sc54-foywl".to_string(),
        registry_subnet_type: 1,
        subnet_kind,
        subnet_kind_source: ClassificationSource::Registry,
        subnet_specialization: SubnetSpecialization::None,
        subnet_specialization_source: ClassificationSource::Computed,
        geographic_scope: GeographicScope::Global,
        geographic_scope_source: ClassificationSource::Computed,
        subnet_label: subnet_kind.as_str().to_string(),
        subnet_label_source: ClassificationSource::Computed,
        node_count: Some(1),
        charges_apply_by_default: subnet_kind.charges_apply_by_default(),
        range_count: 1,
        ranges_shown: 0,
        range_offset: 0,
        range_limit: 1,
        ranges: Vec::new(),
    }
}

#[cfg(feature = "nns-host")]
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

#[cfg(feature = "nns-host")]
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

#[cfg(feature = "nns-host")]
fn sample_nns_node_provider_list_report() -> NnsNodeProviderListReport {
    let provider = sample_nns_node_provider_row();
    NnsNodeProviderListReport {
        schema_version: 1,
        network: "ic".to_string(),
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT.to_string(),
        fetched_by: "ic-query".to_string(),
        node_provider_count: 1,
        node_providers: vec![provider],
    }
}

#[cfg(feature = "nns-host")]
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

#[cfg(feature = "nns-host")]
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

#[cfg(feature = "nns-host")]
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

#[cfg(feature = "nns-host")]
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
        source_endpoint: DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT.to_string(),
        fetched_by: "ic-query".to_string(),
        dry_run: true,
        wrote_cache: false,
        replaced_existing_cache: true,
        node_provider_count: 1,
    }
}

#[cfg(feature = "nns-host")]
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

#[cfg(feature = "nns-host")]
fn sample_nns_proposal_refresh_report(root: &Path) -> NnsProposalRefreshReport {
    NnsProposalRefreshReport {
        schema_version: 1,
        network: "ic".to_string(),
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        proposal_count: 1,
        page_size: 100,
        page_count: 1,
        complete: true,
        replaced_existing_cache: false,
        wrote_cache: false,
        attempt_finalization_error: None,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT.to_string(),
        fetched_by: "ic-query".to_string(),
        cache_path: nns_proposal_cache_path(root, "ic").display().to_string(),
        refresh_attempt_path: nns_proposal_refresh_attempt_path(root, "ic")
            .display()
            .to_string(),
        refresh_lock_path: nns_proposal_refresh_lock_path(root, "ic")
            .display()
            .to_string(),
    }
}

fn sample_nns_node_row() -> NnsNodeRow {
    NnsNodeRow {
        node_principal: "zh3jp-xqaaa-aaaar-qaada-cai".to_string(),
        node_operator_principal: "qoctq-giaaa-aaaar-qaada-cai".to_string(),
        node_provider_principal: "w6gnz-6qaaa-aaaar-qaada-cai".to_string(),
        subnet_principal: "tdb26-jop6g-7sc54-foywl".to_string(),
        subnet_kind: SubnetKind::Application,
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
        topic_text: NnsProposalTopic::Governance,
        status: 4,
        status_text: NnsProposalStatus::Executed,
        reward_status: 3,
        reward_status_text: NnsProposalRewardStatus::Settled,
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
            vote_text: NnsProposalVote::Yes,
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
