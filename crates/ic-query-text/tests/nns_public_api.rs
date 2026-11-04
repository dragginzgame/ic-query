#[cfg(feature = "nns-host")]
use ic_query::nns::NnsInventoryRefreshReport;
#[cfg(feature = "nns-host")]
use ic_query::nns::governance::NnsGovernanceRefreshReport;
#[cfg(feature = "nns-host")]
use ic_query::report::CollectionSourceRequest;
use ic_query::report::SortDirection;
#[cfg(feature = "nns-host")]
use ic_query_text::nns::governance::nns_governance_refresh_report_text;
use std::future::Future;
#[cfg(all(unix, feature = "nns-host"))]
use std::os::unix::fs::PermissionsExt;

#[cfg(feature = "nns-host")]
use ic_query::nns::NnsInventoryError;
#[cfg(feature = "nns-host")]
use ic_query::nns::data_center::DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT;
use ic_query::nns::data_center::NnsDataCenterInfoReport;
use ic_query::nns::data_center::NnsDataCenterListReport;
use ic_query::nns::data_center::NnsDataCenterRow;
#[cfg(feature = "nns-host")]
use ic_query::nns::data_center::NnsDataCenterSource;
#[cfg(feature = "nns-host")]
use ic_query::nns::data_center::build_nns_data_center_info_report_with_source_and_progress;
#[cfg(feature = "nns-host")]
use ic_query::nns::data_center::build_nns_data_center_list_report_with_source_and_progress;
#[cfg(feature = "nns-host")]
use ic_query::nns::data_center::nns_data_center_cache_path;
#[cfg(feature = "nns-host")]
use ic_query::nns::data_center::nns_data_center_refresh_lock_path;
#[cfg(feature = "nns-host")]
use ic_query::nns::data_center::refresh_nns_data_center_report_with_source;
use ic_query_text::nns::data_center::nns_data_center_info_report_text;
use ic_query_text::nns::data_center::nns_data_center_list_report_text;

#[cfg(feature = "nns-host")]
use ic_query::nns::NnsGovernanceCacheRequest;
#[cfg(feature = "nns-host")]
use ic_query::nns::NnsGovernanceRefreshRequest;
use ic_query::nns::NnsInventoryCacheRequest;
use ic_query::nns::NnsInventoryInfoRequest;
use ic_query::nns::NnsInventoryListRequest;
#[cfg(feature = "nns-host")]
use ic_query::nns::NnsInventoryRefreshRequest;
use ic_query::nns::governance::DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT;
use ic_query::nns::governance::NnsGovernanceCollectionStatus;
use ic_query::nns::governance::NnsGovernanceError;
use ic_query::nns::governance::NnsGovernanceMaturityModulation;
use ic_query::nns::governance::NnsGovernanceMaturityModulationReport;
use ic_query::nns::governance::NnsGovernanceReportContext;
use ic_query::nns::governance::NnsGovernanceRequest;
use ic_query::nns::governance::NnsGovernanceSource;
use ic_query::nns::governance::NnsGovernanceSourceData;
use ic_query::nns::governance::NnsGovernanceSourceFuture;
use ic_query::nns::governance::NnsGovernanceSourceProvenance;
use ic_query::nns::governance::NnsGovernanceSourceSelection;
use ic_query::nns::governance::build_nns_governance_economics_report_with_source;
use ic_query::nns::neuron::DEFAULT_NNS_NEURON_SOURCE_ENDPOINT;
use ic_query::nns::neuron::NNS_NEURON_COLLECTION_STATE_SCHEMA_VERSION;
use ic_query::nns::neuron::NNS_NEURON_DISTRIBUTION_REPORT_SCHEMA_VERSION;
use ic_query::nns::neuron::NNS_NEURON_MAX_PAGE_SIZE;
use ic_query::nns::neuron::NnsKnownNeuronData;
use ic_query::nns::neuron::NnsNeuronCollectionState;
use ic_query::nns::neuron::NnsNeuronCollectionStep;
use ic_query::nns::neuron::NnsNeuronDistributionError;
use ic_query::nns::neuron::NnsNeuronDistributionReport;
use ic_query::nns::neuron::NnsNeuronError;
#[cfg(feature = "nns-host")]
use ic_query::nns::neuron::NnsNeuronHostError;
use ic_query::nns::neuron::NnsNeuronInfoRequest;
use ic_query::nns::neuron::NnsNeuronListRequest;
use ic_query::nns::neuron::NnsNeuronPage;
use ic_query::nns::neuron::NnsNeuronRow;
use ic_query::nns::neuron::NnsNeuronSource;
use ic_query::nns::neuron::NnsNeuronSourceFuture;
use ic_query::nns::neuron::NnsNeuronState;
use ic_query::nns::neuron::NnsNeuronStateDistribution;
use ic_query::nns::neuron::NnsNeuronType;
use ic_query::nns::neuron::NnsNeuronTypeDistribution;
use ic_query::nns::neuron::NnsNeuronVisibility;
use ic_query::nns::neuron::NnsNeuronVisibilityDistribution;
use ic_query::nns::neuron::advance_nns_neuron_collection_with_source;
#[cfg(feature = "nns-host")]
use ic_query::nns::neuron::build_nns_neuron_cache_status_report;
use ic_query::nns::neuron::build_nns_neuron_distribution_report;
use ic_query::nns::neuron::build_nns_neuron_info_report_with_source;
use ic_query::nns::neuron::build_nns_neuron_list_report_with_source;
#[cfg(feature = "nns-host")]
use ic_query::nns::neuron::nns_neuron_cache_path;
#[cfg(feature = "nns-host")]
use ic_query::nns::neuron::nns_neuron_refresh_attempt_path;
#[cfg(feature = "nns-host")]
use ic_query::nns::neuron::nns_neuron_refresh_lock_path;
#[cfg(feature = "nns-host")]
use ic_query::nns::node::DEFAULT_NNS_NODE_SOURCE_ENDPOINT;
use ic_query::nns::node::NnsNodeInfoReport;
use ic_query::nns::node::NnsNodeListReport;
#[cfg(feature = "nns-host")]
use ic_query::nns::node::NnsNodeListRequest;
use ic_query::nns::node::NnsNodeRow;
#[cfg(feature = "nns-host")]
use ic_query::nns::node::NnsNodeSource;
#[cfg(feature = "nns-host")]
use ic_query::nns::node::build_nns_node_info_report_with_source_and_progress;
#[cfg(feature = "nns-host")]
use ic_query::nns::node::build_nns_node_list_report_with_source_and_progress;
#[cfg(feature = "nns-host")]
use ic_query::nns::node::nns_node_cache_path;
#[cfg(feature = "nns-host")]
use ic_query::nns::node::nns_node_refresh_lock_path;
#[cfg(feature = "nns-host")]
use ic_query::nns::node::refresh_nns_node_report_with_source;
#[cfg(feature = "nns-host")]
use ic_query::nns::node_operator::DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT;
use ic_query::nns::node_operator::NnsNodeOperatorInfoReport;
use ic_query::nns::node_operator::NnsNodeOperatorListReport;
use ic_query::nns::node_operator::NnsNodeOperatorRow;
#[cfg(feature = "nns-host")]
use ic_query::nns::node_operator::NnsNodeOperatorSource;
#[cfg(feature = "nns-host")]
use ic_query::nns::node_operator::build_nns_node_operator_info_report_with_source_and_progress;
#[cfg(feature = "nns-host")]
use ic_query::nns::node_operator::build_nns_node_operator_list_report_with_source_and_progress;
#[cfg(feature = "nns-host")]
use ic_query::nns::node_operator::nns_node_operator_cache_path;
#[cfg(feature = "nns-host")]
use ic_query::nns::node_operator::nns_node_operator_refresh_lock_path;
#[cfg(feature = "nns-host")]
use ic_query::nns::node_operator::refresh_nns_node_operator_report_with_source;
#[cfg(feature = "nns-host")]
use ic_query::nns::node_provider::DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT;
use ic_query::nns::node_provider::NnsNodeProviderInfoReport;
use ic_query::nns::node_provider::NnsNodeProviderListReport;
use ic_query::nns::node_provider::NnsNodeProviderRow;
#[cfg(feature = "nns-host")]
use ic_query::nns::node_provider::NnsNodeProviderSource;
#[cfg(feature = "nns-host")]
use ic_query::nns::node_provider::build_nns_node_provider_info_report_with_source_and_progress;
#[cfg(feature = "nns-host")]
use ic_query::nns::node_provider::build_nns_node_provider_list_report_with_source_and_progress;
#[cfg(feature = "nns-host")]
use ic_query::nns::node_provider::nns_node_provider_cache_path;
#[cfg(feature = "nns-host")]
use ic_query::nns::node_provider::nns_node_provider_refresh_lock_path;
#[cfg(feature = "nns-host")]
use ic_query::nns::node_provider::refresh_nns_node_provider_report_with_source;
use ic_query::nns::proposals::DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT;
use ic_query::nns::proposals::NNS_PROPOSAL_ACTIVITY_REPORT_SCHEMA_VERSION;
use ic_query::nns::proposals::NNS_PROPOSAL_COLLECTION_STATE_SCHEMA_VERSION;
use ic_query::nns::proposals::NNS_PROPOSAL_MAX_PAGE_SIZE;
use ic_query::nns::proposals::NnsProposalActivityError;
use ic_query::nns::proposals::NnsProposalActivityReport;
use ic_query::nns::proposals::NnsProposalActivityRequest;
use ic_query::nns::proposals::NnsProposalBallotRow;
use ic_query::nns::proposals::NnsProposalCollectionState;
use ic_query::nns::proposals::NnsProposalDayCount;
use ic_query::nns::proposals::NnsProposalError;
#[cfg(feature = "nns-host")]
use ic_query::nns::proposals::NnsProposalHostError;
use ic_query::nns::proposals::NnsProposalListReport;
use ic_query::nns::proposals::NnsProposalListRequest;
use ic_query::nns::proposals::NnsProposalListSort;
use ic_query::nns::proposals::NnsProposalReport;
use ic_query::nns::proposals::NnsProposalRequest;
use ic_query::nns::proposals::NnsProposalRewardStatusCount;
use ic_query::nns::proposals::NnsProposalRewardStatusFilter;
use ic_query::nns::proposals::NnsProposalRow;
use ic_query::nns::proposals::NnsProposalSource;
use ic_query::nns::proposals::NnsProposalSourceFuture;
use ic_query::nns::proposals::NnsProposalStatusCount;
use ic_query::nns::proposals::NnsProposalStatusFilter;
use ic_query::nns::proposals::NnsProposalTally;
use ic_query::nns::proposals::NnsProposalTopicCount;
use ic_query::nns::proposals::NnsProposalTopicFilter;
use ic_query::nns::proposals::advance_nns_proposal_collection_with_source;
use ic_query::nns::proposals::build_nns_proposal_activity_report;
#[cfg(feature = "nns-host")]
use ic_query::nns::proposals::build_nns_proposal_cache_list_report;
#[cfg(feature = "nns-host")]
use ic_query::nns::proposals::build_nns_proposal_cache_status_report;
#[cfg(feature = "nns-host")]
use ic_query::nns::proposals::build_nns_proposal_list_report;
#[cfg(feature = "nns-host")]
use ic_query::nns::proposals::build_nns_proposal_list_report_from_cache;
use ic_query::nns::proposals::build_nns_proposal_list_report_with_source;
#[cfg(feature = "nns-host")]
use ic_query::nns::proposals::build_nns_proposal_report;
#[cfg(feature = "nns-host")]
use ic_query::nns::proposals::build_nns_proposal_report_from_cache;
use ic_query::nns::proposals::build_nns_proposal_report_with_source;
#[cfg(feature = "nns-host")]
use ic_query::nns::proposals::nns_proposal_cache_path;
#[cfg(feature = "nns-host")]
use ic_query::nns::proposals::nns_proposal_cache_root;
#[cfg(feature = "nns-host")]
use ic_query::nns::proposals::nns_proposal_refresh_attempt_path;
#[cfg(feature = "nns-host")]
use ic_query::nns::proposals::nns_proposal_refresh_lock_path;
#[cfg(feature = "nns-host")]
use ic_query::nns::proposals::refresh_nns_proposal_cache;
#[cfg(feature = "nns-host")]
use ic_query::nns::proposals::refresh_nns_proposal_cache_with_source_and_progress;
use ic_query::nns::registry::NnsRegistryCertification;
#[cfg(feature = "nns-host")]
use ic_query::nns::registry::NnsRegistryHostError;
#[cfg(feature = "nns-host")]
use ic_query::nns::registry::NnsRegistrySource;
use ic_query::nns::registry::NnsRegistryVersionReport;
use ic_query::nns::registry::NnsRegistryVersionRequest;
#[cfg(feature = "nns-host")]
use ic_query::nns::registry::build_nns_registry_version_report_with_source;
#[cfg(feature = "nns-host")]
use ic_query::nns::topology::DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT;
use ic_query::nns::topology::NnsTopologyAssessmentStatus;
use ic_query::nns::topology::NnsTopologyCapacityReport;
use ic_query::nns::topology::NnsTopologyCapacityRow;
use ic_query::nns::topology::NnsTopologyCapacityStatus;
use ic_query::nns::topology::NnsTopologyCheckReport;
use ic_query::nns::topology::NnsTopologyCheckRow;
use ic_query::nns::topology::NnsTopologyCoverageReport;
use ic_query::nns::topology::NnsTopologyGapRelationKind;
use ic_query::nns::topology::NnsTopologyGapRow;
use ic_query::nns::topology::NnsTopologyGapSubjectKind;
use ic_query::nns::topology::NnsTopologyGapsReport;
#[cfg(feature = "nns-host")]
use ic_query::nns::topology::NnsTopologyHostError;
use ic_query::nns::topology::NnsTopologyProviderRow;
use ic_query::nns::topology::NnsTopologyProviderStatus;
use ic_query::nns::topology::NnsTopologyProvidersReport;
use ic_query::nns::topology::NnsTopologyRefreshReport;
use ic_query::nns::topology::NnsTopologyRefreshRequest;
use ic_query::nns::topology::NnsTopologyRefreshRow;
#[cfg(feature = "nns-host")]
use ic_query::nns::topology::NnsTopologyRefreshSource;
use ic_query::nns::topology::NnsTopologyRegionRow;
use ic_query::nns::topology::NnsTopologyRegionsReport;
use ic_query::nns::topology::NnsTopologyRegistryVersionRow;
#[cfg(feature = "nns-host")]
use ic_query::nns::topology::NnsTopologySource;
use ic_query::nns::topology::NnsTopologySummaryReport;
use ic_query::nns::topology::NnsTopologyVersionsReport;
#[cfg(feature = "nns-host")]
use ic_query::nns::topology::build_nns_topology_capacity_report_with_source;
#[cfg(feature = "nns-host")]
use ic_query::nns::topology::build_nns_topology_check_report_with_source;
#[cfg(feature = "nns-host")]
use ic_query::nns::topology::build_nns_topology_coverage_report_with_source;
#[cfg(feature = "nns-host")]
use ic_query::nns::topology::build_nns_topology_gaps_report_with_source;
#[cfg(feature = "nns-host")]
use ic_query::nns::topology::build_nns_topology_providers_report_with_source;
#[cfg(feature = "nns-host")]
use ic_query::nns::topology::build_nns_topology_regions_report_with_source;
#[cfg(feature = "nns-host")]
use ic_query::nns::topology::build_nns_topology_summary_report_with_source;
#[cfg(feature = "nns-host")]
use ic_query::nns::topology::build_nns_topology_versions_report_with_source;
#[cfg(feature = "nns-host")]
use ic_query::nns::topology::refresh_nns_topology_report_with_source;
use ic_query::report::ReportResultScope;
#[cfg(feature = "nns-host")]
use ic_query::subnet_catalog::CacheDisposition;
#[cfg(feature = "nns-host")]
use ic_query::subnet_catalog::CatalogAssurance;
#[cfg(feature = "nns-host")]
use ic_query::subnet_catalog::ClassificationSource;
#[cfg(feature = "nns-host")]
use ic_query::subnet_catalog::GeographicScope;
#[cfg(feature = "nns-host")]
use ic_query::subnet_catalog::SubnetCatalogListReport;
#[cfg(feature = "nns-host")]
use ic_query::subnet_catalog::SubnetCatalogRefreshReport;
#[cfg(feature = "nns-host")]
use ic_query::subnet_catalog::SubnetCatalogSubnetRow;
use ic_query::subnet_catalog::SubnetKind;
#[cfg(feature = "nns-host")]
use ic_query::subnet_catalog::SubnetSpecialization;
use ic_query_text::nns::governance::nns_governance_maturity_modulation_report_text;
use ic_query_text::nns::neuron::nns_neuron_distribution_report_text;
use ic_query_text::nns::neuron::nns_neuron_info_report_text;
use ic_query_text::nns::neuron::nns_neuron_list_report_text;
use ic_query_text::nns::node::nns_node_info_report_text;
use ic_query_text::nns::node::nns_node_list_report_text;
use ic_query_text::nns::node_operator::nns_node_operator_info_report_text;
use ic_query_text::nns::node_operator::nns_node_operator_list_report_text;
use ic_query_text::nns::node_provider::nns_node_provider_info_report_text;
use ic_query_text::nns::node_provider::nns_node_provider_list_report_text;
use ic_query_text::nns::proposals::nns_proposal_activity_report_text;
#[cfg(feature = "nns-host")]
use ic_query_text::nns::proposals::nns_proposal_cache_list_report_text;
use ic_query_text::nns::proposals::nns_proposal_list_report_text;
use ic_query_text::nns::proposals::nns_proposal_report_text;
use ic_query_text::nns::registry::nns_registry_version_report_text;
use ic_query_text::nns::topology::nns_topology_capacity_report_text;
use ic_query_text::nns::topology::nns_topology_check_report_text;
use ic_query_text::nns::topology::nns_topology_coverage_report_text;
use ic_query_text::nns::topology::nns_topology_gaps_report_text;
use ic_query_text::nns::topology::nns_topology_providers_report_text;
use ic_query_text::nns::topology::nns_topology_refresh_report_text;
use ic_query_text::nns::topology::nns_topology_regions_report_text;
use ic_query_text::nns::topology::nns_topology_summary_report_text;
use ic_query_text::nns::topology::nns_topology_versions_report_text;
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
    let mut refresh = NnsGovernanceRefreshRequest::new(
        cache.cache_root(),
        &cache.network,
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT,
        1_700_000_000,
        100,
    );
    refresh.max_pages = Some(2);

    assert_eq!(refresh.max_pages, Some(2));

    let query_error = || NnsGovernanceError::AgentCall {
        method: "list_proposals",
        reason: "fixture failure".to_string(),
    };
    assert!(matches!(
        NnsProposalHostError::from(NnsProposalError::from(query_error())),
        NnsProposalHostError::Proposal(NnsProposalError::Governance(
            NnsGovernanceError::AgentCall {
                method: "list_proposals",
                ..
            }
        ))
    ));
    assert!(matches!(
        NnsNeuronHostError::from(NnsNeuronError::from(query_error())),
        NnsNeuronHostError::Neuron(NnsNeuronError::Governance(NnsGovernanceError::AgentCall {
            method: "list_proposals",
            ..
        }))
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
            source: NnsGovernanceSourceProvenance::ReplicaQuery {
                endpoint: DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT.to_string(),
                fetched_by: "fixture".to_string(),
            },
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
    assert_eq!(json["source"]["source_transport"], "replica_query");
    assert_eq!(json["maturity_modulation"]["current_value_permyriad"], -125);
}

#[tokio::test(flavor = "current_thread")]
async fn public_nns_governance_portable_api_accepts_custom_source() {
    let request = NnsGovernanceRequest::replica_query_from_unix_secs(
        "ic",
        DEFAULT_NNS_GOVERNANCE_SOURCE_ENDPOINT,
        1_700_000_000,
        "fixture",
    );
    let report =
        build_nns_governance_economics_report_with_source(&request, &FixtureGovernanceSource)
            .await
            .expect("custom Governance source");

    assert_eq!(report.economics.transaction_fee_e8s, 10_000);
    assert_eq!(report.context.fetched_at, "2023-11-14T22:13:20Z");
}

#[tokio::test(flavor = "current_thread")]
async fn public_nns_governance_builder_rejects_invalid_replica_selection_before_source() {
    let request = NnsGovernanceRequest::replica_query(
        "ic",
        "not-an-absolute-url",
        "2026-08-14T00:00:00Z",
        "fixture",
    );
    let error = build_nns_governance_economics_report_with_source(&request, &PanicGovernanceSource)
        .await
        .expect_err("invalid endpoint must fail");

    assert!(matches!(
        error,
        NnsGovernanceError::InvalidSourceSelection { reason }
            if !reason.is_empty()
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn public_nns_proposal_portable_api_accepts_custom_source() {
    let mut request = NnsProposalListRequest::new(proposal_governance_request(1_700_000_000), 25);
    request.status = NnsProposalStatusFilter::Executed;
    request.reward_status = NnsProposalRewardStatusFilter::Settled;
    request.topic = NnsProposalTopicFilter::Governance;

    let report = build_nns_proposal_list_report_with_source(&request, &FixtureNnsProposalSource)
        .await
        .expect("portable proposal source");
    let detail_request =
        NnsProposalRequest::new(proposal_governance_request(1_700_000_000), 132_411);
    let detail = build_nns_proposal_report_with_source(&detail_request, &FixtureNnsProposalSource)
        .await
        .expect("portable proposal detail source");

    assert_eq!(NNS_PROPOSAL_MAX_PAGE_SIZE, 100);
    assert_eq!(report.proposals.len(), 1);
    assert_eq!(
        report.context.source,
        governance_fixture_provenance(&request.governance)
    );
    assert_eq!(detail.proposal_id, 132_411);
}

#[tokio::test(flavor = "current_thread")]
async fn public_nns_proposal_collection_state_resumes_until_api_exhaustion() {
    let first_request = proposal_governance_request(1_700_000_000);
    let state = NnsProposalCollectionState::new(&first_request, 2, 3)
        .expect("valid proposal collection state");
    let first = advance_nns_proposal_collection_with_source(
        &first_request,
        &state,
        &PagedFixtureNnsProposalSource,
    )
    .await
    .expect("first proposal collection page");

    assert_eq!(
        NNS_PROPOSAL_COLLECTION_STATE_SCHEMA_VERSION,
        first.state.schema_version()
    );
    assert_eq!(
        first.state.status(),
        NnsGovernanceCollectionStatus::Collecting
    );
    assert_eq!(first.state.pages_fetched(), 1);
    let proposals_fetched: u64 = first.state.proposals_fetched();
    assert_eq!(proposals_fetched, 2);
    assert_eq!(first.state.next_before_proposal_id(), Some(2));
    assert_eq!(first.state.network(), "ic");
    assert_eq!(
        first.state.governance_canister_id(),
        "rrkah-fqaaa-aaaaa-aaaaq-cai"
    );
    assert_eq!(first.state.requested_source(), &first_request.source);
    assert_eq!(
        first.state.source(),
        Some(&governance_fixture_provenance(&first_request))
    );
    assert_eq!(first.state.page_size(), 2);
    assert!(!first.state.is_complete());

    let persisted = serde_json::to_string(&first.state).expect("serialize collection state");
    let persisted_json: serde_json::Value =
        serde_json::from_str(&persisted).expect("parse serialized collection state");
    assert_eq!(persisted_json["schema_version"], 1);
    assert_eq!(
        persisted_json["requested_source"]["source_transport"],
        "replica_query"
    );
    let resumed: NnsProposalCollectionState =
        serde_json::from_str(&persisted).expect("deserialize collection state");
    let second_request = proposal_governance_request(1_700_000_100);
    let second = advance_nns_proposal_collection_with_source(
        &second_request,
        &resumed,
        &PagedFixtureNnsProposalSource,
    )
    .await
    .expect("final proposal collection page");

    assert_eq!(second.page.proposals.len(), 1);
    assert_eq!(
        second.state.status(),
        NnsGovernanceCollectionStatus::Complete
    );
    assert_eq!(second.state.pages_fetched(), 2);
    assert_eq!(second.state.proposals_fetched(), 3);
    assert_eq!(second.state.next_before_proposal_id(), None);
    assert_eq!(second.state.started_at(), "2023-11-14T22:13:20Z");
    assert_eq!(second.state.updated_at(), "2023-11-14T22:15:00Z");
    assert!(second.state.is_complete());

    assert_public_nns_proposal_activity(
        &first.page.proposals,
        &second.page.proposals,
        &second.state,
    );

    let restored: NnsProposalCollectionState = serde_json::from_value(
        serde_json::to_value(&second.state).expect("serialize completed state"),
    )
    .expect("restore completed state");
    let error = advance_nns_proposal_collection_with_source(
        &second_request,
        &restored,
        &PagedFixtureNnsProposalSource,
    )
    .await
    .expect_err("complete collection cannot make another source call");
    assert!(matches!(
        error,
        NnsProposalError::CollectionComplete { pages_fetched: 2 }
    ));
}

fn assert_public_nns_proposal_activity(
    first_page: &[NnsProposalRow],
    final_page: &[NnsProposalRow],
    state: &NnsProposalCollectionState,
) {
    let proposals = first_page
        .iter()
        .chain(final_page)
        .cloned()
        .collect::<Vec<_>>();
    let activity: NnsProposalActivityReport = build_nns_proposal_activity_report(
        &NnsProposalActivityRequest::default(),
        state,
        &proposals,
    )
    .expect("portable proposal activity report");
    assert_eq!(
        activity.context.schema_version,
        NNS_PROPOSAL_ACTIVITY_REPORT_SCHEMA_VERSION
    );
    assert_eq!(activity.included_proposal_count, 3);
    assert!(!activity.context.point_in_time_guaranteed);
    let _: &[NnsProposalTopicCount] = &activity.topic_counts;
    let _: &[NnsProposalStatusCount] = &activity.status_counts;
    let _: &[NnsProposalRewardStatusCount] = &activity.reward_status_counts;
    let _: &[NnsProposalDayCount] = &activity.day_counts;
    assert!(nns_proposal_activity_report_text(&activity).contains("topics:"));
    let activity_json = serde_json::to_value(&activity).expect("serialize activity report");
    assert_eq!(activity_json["source"]["source_transport"], "replica_query");

    let invalid_window = NnsProposalActivityRequest {
        from_proposal_timestamp_seconds: Some(10),
        until_proposal_timestamp_seconds: Some(10),
    };
    assert!(matches!(
        build_nns_proposal_activity_report(&invalid_window, state, &proposals),
        Err(NnsProposalActivityError::InvalidTimeWindow { .. })
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn public_nns_proposal_collection_stops_at_explicit_page_limit() {
    let request = proposal_governance_request(1_700_000_000);
    let state = NnsProposalCollectionState::new(&request, 2, 1).expect("bounded collection state");
    let step = advance_nns_proposal_collection_with_source(
        &request,
        &state,
        &PagedFixtureNnsProposalSource,
    )
    .await
    .expect("bounded collection page");

    assert_eq!(
        step.state.status(),
        NnsGovernanceCollectionStatus::PageLimitReached
    );
    assert_eq!(step.state.max_pages(), 1);
    assert!(!step.state.is_complete());
    let restored: NnsProposalCollectionState = serde_json::from_value(
        serde_json::to_value(&step.state).expect("serialize page-limited state"),
    )
    .expect("restore page-limited state");
    let error = advance_nns_proposal_collection_with_source(
        &request,
        &restored,
        &PagedFixtureNnsProposalSource,
    )
    .await
    .expect_err("page-limited collection cannot advance");
    assert!(matches!(
        error,
        NnsProposalError::CollectionPageLimitReached {
            pages_fetched: 1,
            max_pages: 1
        }
    ));

    let changed_request = NnsGovernanceRequest::replica_query_from_unix_secs(
        "ic",
        "https://example.com",
        1_700_000_100,
        "ic-query",
    );
    let error = advance_nns_proposal_collection_with_source(
        &changed_request,
        &state,
        &PagedFixtureNnsProposalSource,
    )
    .await
    .expect_err("collection identity must be fixed before source invocation");
    assert!(matches!(
        error,
        NnsProposalError::CollectionRequestMismatch {
            field: "requested_source",
            ..
        }
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn public_nns_proposal_collection_rejects_a_changed_canister_collector() {
    let first_request =
        NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 1_700_000_000);
    let state = NnsProposalCollectionState::new(&first_request, 2, 3)
        .expect("valid canister collection state");
    let first = advance_nns_proposal_collection_with_source(
        &first_request,
        &state,
        &ChangingCanisterProposalSource,
    )
    .await
    .expect("first canister proposal page");
    let second_request =
        NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 1_700_000_100);

    let error = advance_nns_proposal_collection_with_source(
        &second_request,
        &first.state,
        &ChangingCanisterProposalSource,
    )
    .await
    .expect_err("collector principal must remain fixed");
    assert!(matches!(
        error,
        NnsProposalError::CollectionSourceChanged { .. }
    ));
}

struct FixtureGovernanceSource;

struct PanicGovernanceSource;

impl NnsGovernanceSource for FixtureGovernanceSource {
    fn fetch_economics<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, ic_query::nns::governance::NnsGovernanceEconomics> {
        Box::pin(async move {
            Ok(NnsGovernanceSourceData::new(
                ic_query::nns::governance::NnsGovernanceEconomics {
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
                },
                governance_fixture_provenance(request),
            ))
        })
    }

    fn fetch_metrics<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, ic_query::nns::governance::NnsGovernanceMetrics> {
        Box::pin(async { unreachable!("not used by this public API fixture") })
    }

    fn fetch_reward_event<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, ic_query::nns::governance::NnsGovernanceRewardEvent> {
        Box::pin(async { unreachable!("not used by this public API fixture") })
    }

    fn fetch_maturity_modulation<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, Option<NnsGovernanceMaturityModulation>> {
        Box::pin(async { unreachable!("not used by this public API fixture") })
    }
}

impl NnsGovernanceSource for PanicGovernanceSource {
    fn fetch_economics<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, ic_query::nns::governance::NnsGovernanceEconomics> {
        panic!("source must not be called")
    }

    fn fetch_metrics<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, ic_query::nns::governance::NnsGovernanceMetrics> {
        panic!("source must not be called")
    }

    fn fetch_reward_event<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, ic_query::nns::governance::NnsGovernanceRewardEvent> {
        panic!("source must not be called")
    }

    fn fetch_maturity_modulation<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
    ) -> NnsGovernanceSourceFuture<'a, Option<NnsGovernanceMaturityModulation>> {
        panic!("source must not be called")
    }
}

#[tokio::test(flavor = "current_thread")]
async fn governance_rejects_invalid_request_timestamps_before_source_calls() {
    for mut request in [
        NnsGovernanceRequest::replica_query_from_unix_secs(
            "ic",
            "https://example.com",
            1_700_000_000,
            "fixture",
        ),
        NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 1_700_000_000),
    ] {
        for timestamp in [
            "invalid",
            "2026-02-29T00:00:00Z",
            "2026-6-04T00:00:00Z",
            "2026-06-04T0:00:00Z",
            "+2026-06-04T00:00:00Z",
            "9223372036854775807-01-01T00:00:00Z",
        ] {
            request.fetched_at = timestamp.to_string();
            assert!(matches!(
                run_ready(build_nns_governance_economics_report_with_source(
                    &request,
                    &PanicGovernanceSource
                )),
                Err(NnsGovernanceError::InvalidTimestamp {
                    field: "fetched_at",
                    value,
                    ..
                }) if value == timestamp
            ));
        }
    }
}

fn governance_fixture_provenance(request: &NnsGovernanceRequest) -> NnsGovernanceSourceProvenance {
    let NnsGovernanceSourceSelection::ReplicaQuery {
        endpoint,
        fetched_by,
    } = &request.source
    else {
        panic!("fixture requires a replica query")
    };
    NnsGovernanceSourceProvenance::ReplicaQuery {
        endpoint: endpoint.clone(),
        fetched_by: fetched_by.clone(),
    }
}

fn proposal_governance_request(now_unix_secs: u64) -> NnsGovernanceRequest {
    NnsGovernanceRequest::replica_query_from_unix_secs(
        "ic",
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT,
        now_unix_secs,
        "ic-query",
    )
}

fn governance_report_context(request: &NnsGovernanceRequest) -> NnsGovernanceReportContext {
    NnsGovernanceReportContext {
        schema_version: 1,
        network: request.network.clone(),
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        fetched_at: request.fetched_at.clone(),
        source: governance_fixture_provenance(request),
    }
}

fn run_ready<T>(future: impl Future<Output = T>) -> T {
    use std::{
        pin::pin,
        task::{Context, Poll, Waker},
    };

    let mut future = pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("fixture future unexpectedly returned Pending"),
    }
}

#[test]
fn public_nns_neuron_api_is_constructible_and_renderable() {
    let mut request = NnsNeuronListRequest::new(neuron_governance_request(1_700_000_000), 25);
    request.exclusive_start_neuron_id = Some(10);

    assert_eq!(request.limit, 25);
    assert_eq!(request.exclusive_start_neuron_id, Some(10));
    assert_eq!(NNS_NEURON_MAX_PAGE_SIZE, 300);

    let row = sample_public_neuron(11);
    assert_eq!(
        NnsNeuronState::from_code(row.state),
        NnsNeuronState::NotDissolving
    );
    assert_eq!(
        NnsNeuronVisibility::from_code(row.visibility),
        NnsNeuronVisibility::Public
    );
    assert_eq!(
        NnsNeuronType::from_code(row.neuron_type),
        NnsNeuronType::Unknown
    );
    let report = ic_query::nns::neuron::NnsNeuronListReport {
        context: governance_report_context(&request.governance),
        provenance: ic_query::report::ReportProvenance::Live,
        requested_limit: request.limit,
        exclusive_start_neuron_id: request.exclusive_start_neuron_id,
        next_start_neuron_id: None,
        total_neuron_count: None,
        point_in_time_guaranteed: false,

        neurons: vec![row.clone()],
    };
    assert!(
        nns_neuron_list_report_text(&report, ic_query_text::TextOptions { verbose: true })
            .contains("Neuron 11")
    );
    let report_json = serde_json::to_value(&report).expect("serialize NNS neuron list report");
    assert_eq!(report_json["schema_version"], 1);
    assert_eq!(report_json["source"]["source_transport"], "replica_query");
    assert_eq!(report_json["neurons"][0]["state"], 1);
    assert_eq!(report_json["neurons"][0]["visibility"], 2);
    assert_eq!(
        report_json["neurons"][0]["neuron_type"],
        serde_json::Value::Null
    );
    assert_eq!(
        serde_json::to_value(ic_query::nns::governance::NnsGovernanceVote::Unknown(99))
            .expect("serialize unknown neuron vote"),
        "unknown(99)"
    );

    let info = ic_query::nns::neuron::NnsNeuronInfoReport {
        context: governance_report_context(&request.governance),
        provenance: ic_query::report::ReportProvenance::Live,

        neuron: row,
    };
    let text = nns_neuron_info_report_text(&info);
    assert!(text.contains("STAKE ICP"));
    assert!(text.contains("11"));
    assert_eq!(
        serde_json::to_value(&info).unwrap()["neuron"]["neuron_id"],
        11
    );
}

#[tokio::test(flavor = "current_thread")]
async fn public_nns_neuron_portable_api_accepts_custom_source() {
    let list_request = NnsNeuronListRequest::new(neuron_governance_request(1_700_000_000), 1);
    let list = build_nns_neuron_list_report_with_source(&list_request, &FixtureNnsNeuronSource)
        .await
        .expect("custom neuron source");
    assert_eq!(list.neurons[0].neuron_id, 7);
    assert_eq!(
        list.context.source,
        governance_fixture_provenance(&list_request.governance)
    );

    let info_request = NnsNeuronInfoRequest::new(neuron_governance_request(1_700_000_000), 7);
    let info = build_nns_neuron_info_report_with_source(&info_request, &FixtureNnsNeuronSource)
        .await
        .expect("custom neuron detail");
    assert_eq!(info.neuron.neuron_id, 7);

    let invalid_request = NnsNeuronListRequest::new(
        neuron_governance_request(1_700_000_000),
        NNS_NEURON_MAX_PAGE_SIZE + 1,
    );
    let error = build_nns_neuron_list_report_with_source(&invalid_request, &FixtureNnsNeuronSource)
        .await
        .expect_err("oversized neuron page must fail");
    assert!(matches!(error, NnsNeuronError::InvalidPageSize { .. }));
}

#[tokio::test(flavor = "current_thread")]
async fn public_nns_neuron_collection_state_resumes_until_api_exhaustion() {
    let first_request = neuron_governance_request(1_700_000_000);
    let state =
        NnsNeuronCollectionState::new(&first_request, 2, 3).expect("valid neuron collection state");
    let first = advance_nns_neuron_collection_with_source(
        &first_request,
        &state,
        &PagedFixtureNnsNeuronSource,
    )
    .await
    .expect("first neuron collection page");

    assert_eq!(
        NNS_NEURON_COLLECTION_STATE_SCHEMA_VERSION,
        first.state.schema_version()
    );
    assert_eq!(
        first.state.status(),
        NnsGovernanceCollectionStatus::Collecting
    );
    assert_eq!(first.state.pages_fetched(), 1);
    let neurons_fetched: u64 = first.state.neurons_fetched();
    assert_eq!(neurons_fetched, 2);
    assert_eq!(first.state.next_start_neuron_id(), Some(2));
    assert_eq!(first.state.network(), "ic");
    assert_eq!(
        first.state.governance_canister_id(),
        "rrkah-fqaaa-aaaaa-aaaaq-cai"
    );
    assert_eq!(first.state.requested_source(), &first_request.source);
    assert_eq!(
        first.state.source(),
        Some(&governance_fixture_provenance(&first_request))
    );
    assert_eq!(first.state.page_size(), 2);
    assert!(!first.state.is_complete());

    let persisted = serde_json::to_string(&first).expect("serialize collection step");
    let persisted_json: serde_json::Value =
        serde_json::from_str(&persisted).expect("parse serialized collection step");
    assert_eq!(persisted_json["state"]["schema_version"], 1);
    assert_eq!(
        persisted_json["state"]["requested_source"]["source_transport"],
        "replica_query"
    );
    let resumed: NnsNeuronCollectionStep =
        serde_json::from_str(&persisted).expect("deserialize collection step");
    let second_request = neuron_governance_request(1_700_000_100);
    let second = advance_nns_neuron_collection_with_source(
        &second_request,
        &resumed.state,
        &PagedFixtureNnsNeuronSource,
    )
    .await
    .expect("final neuron collection page");

    assert_eq!(second.page.neurons.len(), 1);
    assert_eq!(
        second.state.status(),
        NnsGovernanceCollectionStatus::Complete
    );
    assert_eq!(second.state.pages_fetched(), 2);
    assert_eq!(second.state.neurons_fetched(), 3);
    assert_eq!(second.state.next_start_neuron_id(), None);
    assert_eq!(second.state.started_at(), "2023-11-14T22:13:20Z");
    assert_eq!(second.state.updated_at(), "2023-11-14T22:15:00Z");
    assert!(second.state.is_complete());

    assert_public_nns_neuron_distribution(&first.page.neurons, &second.page.neurons, &second.state);

    let restored: NnsNeuronCollectionState = serde_json::from_value(
        serde_json::to_value(&second.state).expect("serialize completed state"),
    )
    .expect("restore completed state");
    let error = advance_nns_neuron_collection_with_source(
        &second_request,
        &restored,
        &PagedFixtureNnsNeuronSource,
    )
    .await
    .expect_err("complete collection cannot make another source call");
    assert!(matches!(
        error,
        NnsNeuronError::CollectionComplete { pages_fetched: 2 }
    ));
}

fn assert_public_nns_neuron_distribution(
    first_page: &[NnsNeuronRow],
    final_page: &[NnsNeuronRow],
    state: &NnsNeuronCollectionState,
) {
    let neurons = first_page
        .iter()
        .chain(final_page)
        .cloned()
        .collect::<Vec<_>>();
    let distribution: NnsNeuronDistributionReport =
        build_nns_neuron_distribution_report(state, &neurons)
            .expect("portable public-neuron distribution");
    assert_eq!(
        distribution.context.schema_version,
        NNS_NEURON_DISTRIBUTION_REPORT_SCHEMA_VERSION
    );
    assert_eq!(distribution.collected_neuron_count, 3);
    assert_eq!(distribution.total_effective_stake_e8s, 300_000_000);
    assert!(!distribution.context.point_in_time_guaranteed);
    let _: &[NnsNeuronStateDistribution] = &distribution.state_distribution;
    let _: &[NnsNeuronVisibilityDistribution] = &distribution.visibility_distribution;
    let _: &[NnsNeuronTypeDistribution] = &distribution.neuron_type_distribution;
    assert!(nns_neuron_distribution_report_text(&distribution).contains("states:"));
    let json = serde_json::to_value(&distribution).expect("serialize neuron distribution");
    assert_eq!(json["source"]["source_transport"], "replica_query");

    assert!(matches!(
        build_nns_neuron_distribution_report(state, &neurons[..2]),
        Err(NnsNeuronDistributionError::NeuronCountMismatch { .. })
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn public_nns_neuron_collection_stops_at_explicit_page_limit() {
    let request = neuron_governance_request(1_700_000_000);
    assert!(matches!(
        NnsNeuronCollectionState::new(&request, 2, 0),
        Err(NnsNeuronError::InvalidCollectionMaxPages)
    ));
    let state = NnsNeuronCollectionState::new(&request, 2, 1).expect("bounded collection state");
    let step =
        advance_nns_neuron_collection_with_source(&request, &state, &PagedFixtureNnsNeuronSource)
            .await
            .expect("bounded collection page");

    assert_eq!(
        step.state.status(),
        NnsGovernanceCollectionStatus::PageLimitReached
    );
    assert_eq!(step.state.max_pages(), 1);
    assert!(!step.state.is_complete());
    let restored: NnsNeuronCollectionState = serde_json::from_value(
        serde_json::to_value(&step.state).expect("serialize page-limited state"),
    )
    .expect("restore page-limited state");
    let error = advance_nns_neuron_collection_with_source(
        &request,
        &restored,
        &PagedFixtureNnsNeuronSource,
    )
    .await
    .expect_err("page-limited collection cannot advance");
    assert!(matches!(
        error,
        NnsNeuronError::CollectionPageLimitReached {
            pages_fetched: 1,
            max_pages: 1
        }
    ));

    let changed_request = NnsGovernanceRequest::replica_query_from_unix_secs(
        "ic",
        "https://example.com",
        1_700_000_100,
        "ic-query",
    );
    let error = advance_nns_neuron_collection_with_source(
        &changed_request,
        &state,
        &PagedFixtureNnsNeuronSource,
    )
    .await
    .expect_err("collection identity must be fixed before source invocation");
    assert!(matches!(
        error,
        NnsNeuronError::CollectionRequestMismatch {
            field: "requested_source",
            ..
        }
    ));

    let mut invalid_json = serde_json::to_value(&step.state).expect("serialize collection state");
    invalid_json["next_start_neuron_id"] = serde_json::Value::Null;
    let invalid_state: NnsNeuronCollectionState =
        serde_json::from_value(invalid_json).expect("deserialize modified collection state");
    let error = advance_nns_neuron_collection_with_source(
        &request,
        &invalid_state,
        &PagedFixtureNnsNeuronSource,
    )
    .await
    .expect_err("a full page cannot be changed into false completion");
    assert!(matches!(
        error,
        NnsNeuronError::InvalidCollectionState { .. }
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn public_nns_neuron_collection_rejects_a_changed_canister_collector() {
    let first_request =
        NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 1_700_000_000);
    let state = NnsNeuronCollectionState::new(&first_request, 2, 3)
        .expect("valid canister collection state");
    let first = advance_nns_neuron_collection_with_source(
        &first_request,
        &state,
        &ChangingCanisterNeuronSource,
    )
    .await
    .expect("first canister neuron page");
    let second_request =
        NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 1_700_000_100);

    let error = advance_nns_neuron_collection_with_source(
        &second_request,
        &first.state,
        &ChangingCanisterNeuronSource,
    )
    .await
    .expect_err("collector principal must remain fixed");
    assert!(matches!(
        error,
        NnsNeuronError::CollectionSourceChanged { .. }
    ));
}

#[cfg(feature = "nns-host")]
#[test]
fn public_nns_neuron_host_api_exposes_cache_requests() {
    let root = temp_root("public-neuron-api");
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

struct FixtureNnsNeuronSource;

struct PagedFixtureNnsNeuronSource;

struct ChangingCanisterNeuronSource;

impl NnsNeuronSource for FixtureNnsNeuronSource {
    fn fetch_neuron_page<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        _exclusive_start_neuron_id: Option<u64>,
        page_size: u32,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronPage> {
        Box::pin(async move {
            let neurons = vec![sample_public_neuron(7)];
            Ok(NnsGovernanceSourceData::new(
                NnsNeuronPage {
                    next_start_neuron_id: (page_size == 1).then_some(7),
                    neurons,
                },
                governance_fixture_provenance(request),
            ))
        })
    }

    fn fetch_neuron<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        neuron_id: u64,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronRow> {
        Box::pin(async move {
            Ok(NnsGovernanceSourceData::new(
                sample_public_neuron(neuron_id),
                governance_fixture_provenance(request),
            ))
        })
    }
}

impl NnsNeuronSource for PagedFixtureNnsNeuronSource {
    fn fetch_neuron_page<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        exclusive_start_neuron_id: Option<u64>,
        page_size: u32,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronPage> {
        Box::pin(async move {
            assert_eq!(page_size, 2);
            let (neurons, next_start_neuron_id) = match exclusive_start_neuron_id {
                None => (
                    vec![sample_public_neuron(1), sample_public_neuron(2)],
                    Some(2),
                ),
                Some(2) => (vec![sample_public_neuron(3)], None),
                other => panic!("unexpected neuron collection cursor: {other:?}"),
            };
            Ok(NnsGovernanceSourceData::new(
                NnsNeuronPage {
                    neurons,
                    next_start_neuron_id,
                },
                governance_fixture_provenance(request),
            ))
        })
    }

    fn fetch_neuron<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
        _neuron_id: u64,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronRow> {
        panic!("neuron collection does not fetch exact detail")
    }
}

impl NnsNeuronSource for ChangingCanisterNeuronSource {
    fn fetch_neuron_page<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
        exclusive_start_neuron_id: Option<u64>,
        page_size: u32,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronPage> {
        Box::pin(async move {
            assert_eq!(page_size, 2);
            let (neurons, next_start_neuron_id, collector_canister_id) =
                match exclusive_start_neuron_id {
                    None => (
                        vec![sample_public_neuron(1), sample_public_neuron(2)],
                        Some(2),
                        "aaaaa-aa",
                    ),
                    Some(2) => (vec![sample_public_neuron(3)], None, "2vxsx-fae"),
                    other => panic!("unexpected neuron collection cursor: {other:?}"),
                };
            Ok(NnsGovernanceSourceData::new(
                NnsNeuronPage {
                    neurons,
                    next_start_neuron_id,
                },
                NnsGovernanceSourceProvenance::ReplicatedInterCanisterCall {
                    collector_canister_id: collector_canister_id.to_string(),
                },
            ))
        })
    }

    fn fetch_neuron<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
        _neuron_id: u64,
    ) -> NnsNeuronSourceFuture<'a, NnsNeuronRow> {
        panic!("neuron collection does not fetch exact detail")
    }
}

fn neuron_governance_request(now_unix_secs: u64) -> NnsGovernanceRequest {
    NnsGovernanceRequest::replica_query_from_unix_secs(
        "ic",
        DEFAULT_NNS_NEURON_SOURCE_ENDPOINT,
        now_unix_secs,
        "ic-query",
    )
}

fn sample_public_neuron(neuron_id: u64) -> NnsNeuronRow {
    NnsNeuronRow {
        neuron_id,
        state: 1,
        visibility: Some(2),

        neuron_type: None,
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
        schema_version: 1,
        provenance: ic_query::nns::registry::NnsRegistryInventoryProvenance {
            network: request.network,
            source_endpoint: request.source_endpoint,
            ..sample_registry_provenance()
        },
        certification: public_registry_certification(),
    };

    let text = nns_registry_version_report_text(&report);
    let json = serde_json::to_value(&report).expect("Registry version JSON");
    for (key, value) in serde_json::to_value(&report.provenance)
        .unwrap()
        .as_object()
        .unwrap()
    {
        assert_eq!(
            json.get(key),
            Some(value),
            "flat Registry provenance field {key}"
        );
    }
    let decoded: NnsRegistryVersionReport = serde_json::from_value(json).unwrap();
    assert_eq!(decoded, report);

    assert!(text.contains("network: ic"));
    assert!(text.contains("registry_version: 42"));
}

#[cfg(feature = "nns-host")]
#[tokio::test(flavor = "current_thread")]
async fn public_nns_registry_host_api_accepts_custom_source_adapter() {
    let request = NnsRegistryVersionRequest::new("ic", "https://icp-api.io", 1_700_000_000);
    let report = build_nns_registry_version_report_with_source(&request, &FixtureNnsRegistrySource)
        .await
        .expect("registry version report");

    assert_eq!(report.provenance.network, "ic");
    assert_eq!(
        report.provenance.registry_canister_id,
        "rwlgt-iiaaa-aaaaa-aaaaa-cai"
    );
    assert_eq!(report.provenance.registry_version, 42);
    assert_eq!(report.provenance.source_endpoint, "https://icp-api.io");
}

#[cfg(feature = "nns-host")]
struct FixtureNnsRegistrySource;

#[cfg(feature = "nns-host")]
impl NnsRegistrySource for FixtureNnsRegistrySource {
    fn fetch_registry_version<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> ic_query::report::SourceFuture<
        'a,
        Result<ic_query::nns::registry::NnsRegistryVersionData, NnsRegistryHostError>,
    > {
        Box::pin(async move {
            assert_eq!(request.endpoint, "https://icp-api.io");
            assert_eq!(request.fetched_by, "ic-query");
            assert_ne!(request.fetched_at, "");

            Ok(ic_query::nns::registry::NnsRegistryVersionData {
                provenance: ic_query::nns::registry::NnsRegistryInventoryProvenance {
                    fetched_at: request.fetched_at.clone(),
                    source_endpoint: request.endpoint.clone(),
                    fetched_by: request.fetched_by.clone(),
                    ..sample_registry_provenance()
                },
                certification: public_registry_certification(),
            })
        })
    }
}

fn public_registry_certification() -> NnsRegistryCertification {
    NnsRegistryCertification {
        certificate_verified: true,
        certificate_time_nanos: 1_700_000_000_000_000_000,

        root_key_digest: "ab".repeat(32),
        certificate_hex: "cd".repeat(8),

        hash_tree_hex: "ef".repeat(4),
    }
}

#[test]
fn public_nns_node_api_is_constructible_and_renderable() {
    let cache = NnsInventoryCacheRequest::new(".", "ic");
    let list_report = sample_nns_node_list_report();
    let node = list_report.nodes[0].clone();

    let list_text = nns_node_list_report_text(&list_report, ic_query_text::TextOptions::default());
    let verbose_text =
        nns_node_list_report_text(&list_report, ic_query_text::TextOptions { verbose: true });

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
        provenance: ic_query::nns::registry::NnsRegistryInventoryProvenance {
            network: info_request.cache.network,
            source_endpoint: info_request.source_endpoint,
            ..sample_registry_provenance()
        },
        node,
    };

    let info_text = nns_node_info_report_text(&info_report);

    assert!(info_text.contains("resolved_from: node_principal"));
    assert!(info_text.contains("data_center_id: zh1"));
}

#[test]
fn public_nns_data_center_api_is_constructible_and_renderable() {
    let cache = NnsInventoryCacheRequest::new(".", "ic");
    let list = sample_nns_data_center_list_report();
    let data_center = list.data_centers[0].clone();

    let text = nns_data_center_list_report_text(&list, ic_query_text::TextOptions::default());
    let verbose_text =
        nns_data_center_list_report_text(&list, ic_query_text::TextOptions { verbose: true });

    assert!(text.contains("data_centers: ic count 1"));
    assert!(text.contains("Zurich"));
    assert!(verbose_text.contains("registry_version:"));

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
        provenance: ic_query::nns::registry::NnsRegistryInventoryProvenance {
            network: info_request.cache.network,
            source_endpoint: info_request.source_endpoint,
            ..sample_registry_provenance()
        },
        data_center,
    };

    let info_text = nns_data_center_info_report_text(&info);

    assert!(info_text.contains("resolved_from: data_center_id"));
    assert!(info_text.contains("node_count: 12"));
}

#[test]
fn public_nns_node_provider_api_is_constructible_and_renderable() {
    let cache = NnsInventoryCacheRequest::new(".", "ic");
    let list = sample_nns_node_provider_list_report();
    let provider = list.node_providers[0].clone();

    let text = nns_node_provider_list_report_text(&list, ic_query_text::TextOptions::default());
    let verbose_text =
        nns_node_provider_list_report_text(&list, ic_query_text::TextOptions { verbose: true });

    assert!(text.contains("node_providers: ic count 1"));
    assert!(text.contains("12"));
    assert!(verbose_text.contains(&provider.node_provider_principal));
    assert_eq!(
        serde_json::to_value(&list).unwrap()["node_providers"][0]["reward_account_hex"],
        "deadbeef"
    );

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
        provenance: ic_query::nns::registry::NnsRegistryInventoryProvenance {
            network: info_request.cache.network,
            source_endpoint: info_request.source_endpoint,
            ..sample_registry_provenance()
        },
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        node_provider: provider,
    };

    let info_text = nns_node_provider_info_report_text(&info);

    assert!(info_text.contains("resolved_from: node_provider_principal"));
    assert!(info_text.contains("reward_account_hex: deadbeef"));
}

#[test]
fn public_nns_node_operator_api_is_constructible_and_renderable() {
    let cache = NnsInventoryCacheRequest::new(".", "ic");
    let list = sample_nns_node_operator_list_report();
    let operator = list.node_operators[0].clone();

    let text = nns_node_operator_list_report_text(&list, ic_query_text::TextOptions::default());
    let verbose_text =
        nns_node_operator_list_report_text(&list, ic_query_text::TextOptions { verbose: true });

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
        provenance: ic_query::nns::registry::NnsRegistryInventoryProvenance {
            network: info_request.cache.network,
            source_endpoint: info_request.source_endpoint,
            ..sample_registry_provenance()
        },
        node_operator: operator,
    };

    let info_text = nns_node_operator_info_report_text(&info);

    assert!(info_text.contains("resolved_from: node_operator_principal"));
    assert!(info_text.contains("node_allowance: 28"));
}

#[cfg(feature = "nns-host")]
#[tokio::test(flavor = "current_thread")]
async fn public_nns_inventory_host_api_accepts_custom_source_adapters() {
    let root = temp_root("nns-inventory-source-public-api");

    assert_public_nns_node_custom_source_api(&root).await;
    assert_public_nns_data_center_custom_source_api(&root).await;
    assert_public_nns_node_provider_custom_source_api(&root).await;
    assert_public_nns_node_operator_custom_source_api(&root).await;

    let _ = fs::remove_dir_all(root);
}

#[cfg(feature = "nns-host")]
async fn assert_public_nns_node_custom_source_api(root: &Path) {
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
    let mut node_refresh_request = NnsInventoryRefreshRequest::new(
        node_cache,
        DEFAULT_NNS_NODE_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    node_refresh_request.dry_run = true;

    let node_list = build_nns_node_list_report_with_source_and_progress(
        &node_list_request,
        &FixtureNnsNodeSource,
        &mut |_| {},
    )
    .await
    .expect("node list report");
    let node_info = build_nns_node_info_report_with_source_and_progress(
        &node_info_request,
        &FixtureNnsNodeSource,
        &mut |_| {},
    )
    .await
    .expect("node info report");
    let node_refresh =
        refresh_nns_node_report_with_source(&node_refresh_request, &FixtureNnsNodeSource)
            .await
            .expect("node refresh report");

    assert_eq!(node_list.nodes.len(), 1);
    assert_eq!(
        node_info.node.node_principal,
        sample_nns_node_row().node_principal
    );
    assert_eq!(node_refresh.row_count, 1);
    assert_eq!(node_refresh.collection, "nodes");
    assert_eq!(node_refresh.schema_version, 1);
}

#[cfg(feature = "nns-host")]
async fn assert_public_nns_data_center_custom_source_api(root: &Path) {
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
    let mut data_center_refresh_request = NnsInventoryRefreshRequest::new(
        data_center_cache,
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    data_center_refresh_request.dry_run = true;

    let data_center_list = build_nns_data_center_list_report_with_source_and_progress(
        &data_center_list_request,
        &FixtureNnsDataCenterSource,
        &mut |_| {},
    )
    .await
    .expect("data-center list report");
    let data_center_info = build_nns_data_center_info_report_with_source_and_progress(
        &data_center_info_request,
        &FixtureNnsDataCenterSource,
        &mut |_| {},
    )
    .await
    .expect("data-center info report");
    let data_center_refresh = refresh_nns_data_center_report_with_source(
        &data_center_refresh_request,
        &FixtureNnsDataCenterSource,
    )
    .await
    .expect("data-center refresh report");

    assert_eq!(data_center_list.data_centers.len(), 1);
    assert_eq!(
        data_center_info.data_center.data_center_id,
        sample_nns_data_center_row().data_center_id
    );
    assert_eq!(data_center_refresh.row_count, 1);
    assert_eq!(data_center_refresh.collection, "data_centers");
    assert_eq!(data_center_refresh.schema_version, 1);
}

#[cfg(feature = "nns-host")]
async fn assert_public_nns_node_provider_custom_source_api(root: &Path) {
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
    let mut node_provider_refresh_request = NnsInventoryRefreshRequest::new(
        node_provider_cache,
        DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    node_provider_refresh_request.dry_run = true;

    let node_provider_list = build_nns_node_provider_list_report_with_source_and_progress(
        &node_provider_list_request,
        &FixtureNnsNodeProviderSource,
        &mut |_| {},
    )
    .await
    .expect("node-provider list report");
    let node_provider_info = build_nns_node_provider_info_report_with_source_and_progress(
        &node_provider_info_request,
        &FixtureNnsNodeProviderSource,
        &mut |_| {},
    )
    .await
    .expect("node-provider info report");
    let node_provider_refresh = refresh_nns_node_provider_report_with_source(
        &node_provider_refresh_request,
        &FixtureNnsNodeProviderSource,
    )
    .await
    .expect("node-provider refresh report");

    assert_eq!(node_provider_list.node_providers.len(), 1);
    assert_eq!(
        node_provider_info.node_provider.node_provider_principal,
        sample_nns_node_provider_row().node_provider_principal
    );
    assert_eq!(node_provider_refresh.row_count, 1);
    assert_eq!(node_provider_refresh.collection, "node_providers");
    assert_eq!(node_provider_refresh.schema_version, 1);
}

#[cfg(feature = "nns-host")]
async fn assert_public_nns_node_operator_custom_source_api(root: &Path) {
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
    let mut node_operator_refresh_request = NnsInventoryRefreshRequest::new(
        node_operator_cache,
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        1_700_000_000,
    );
    node_operator_refresh_request.dry_run = true;

    let node_operator_list = build_nns_node_operator_list_report_with_source_and_progress(
        &node_operator_list_request,
        &FixtureNnsNodeOperatorSource,
        &mut |_| {},
    )
    .await
    .expect("node-operator list report");
    let node_operator_info = build_nns_node_operator_info_report_with_source_and_progress(
        &node_operator_info_request,
        &FixtureNnsNodeOperatorSource,
        &mut |_| {},
    )
    .await
    .expect("node-operator info report");
    let node_operator_refresh = refresh_nns_node_operator_report_with_source(
        &node_operator_refresh_request,
        &FixtureNnsNodeOperatorSource,
    )
    .await
    .expect("node-operator refresh report");

    assert_eq!(node_operator_list.node_operators.len(), 1);
    assert_eq!(
        node_operator_info.node_operator.node_operator_principal,
        sample_nns_node_operator_row().node_operator_principal
    );
    assert_eq!(node_operator_refresh.row_count, 1);
    assert_eq!(node_operator_refresh.collection, "node_operators");
    assert_eq!(node_operator_refresh.schema_version, 1);
}

#[cfg(feature = "nns-host")]
struct FixtureNnsNodeSource;

#[cfg(feature = "nns-host")]
impl NnsNodeSource for FixtureNnsNodeSource {
    fn fetch_node_list_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<NnsNodeListReport, NnsInventoryError>> {
        Box::pin(async move {
            assert_inventory_source_request(
                &request.network,
                &request.endpoint,
                &request.fetched_at,
                &request.fetched_by,
            );
            let mut report = sample_nns_node_list_report();
            report.provenance.network.clone_from(&request.network);
            report.provenance.fetched_at.clone_from(&request.fetched_at);
            report
                .provenance
                .source_endpoint
                .clone_from(&request.endpoint);
            report.provenance.fetched_by.clone_from(&request.fetched_by);
            Ok(report)
        })
    }
}

#[cfg(feature = "nns-host")]
struct FixtureNnsDataCenterSource;

#[cfg(feature = "nns-host")]
impl NnsDataCenterSource for FixtureNnsDataCenterSource {
    fn fetch_data_center_list_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<NnsDataCenterListReport, NnsInventoryError>>
    {
        Box::pin(async move {
            assert_inventory_source_request(
                &request.network,
                &request.endpoint,
                &request.fetched_at,
                &request.fetched_by,
            );
            let mut report = sample_nns_data_center_list_report();
            report.provenance.network.clone_from(&request.network);
            report.provenance.fetched_at.clone_from(&request.fetched_at);
            report
                .provenance
                .source_endpoint
                .clone_from(&request.endpoint);
            report.provenance.fetched_by.clone_from(&request.fetched_by);
            Ok(report)
        })
    }
}

#[cfg(feature = "nns-host")]
struct FixtureNnsNodeProviderSource;

#[cfg(feature = "nns-host")]
impl NnsNodeProviderSource for FixtureNnsNodeProviderSource {
    fn fetch_node_provider_list_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<NnsNodeProviderListReport, NnsInventoryError>>
    {
        Box::pin(async move {
            assert_inventory_source_request(
                &request.network,
                &request.endpoint,
                &request.fetched_at,
                &request.fetched_by,
            );
            let mut report = sample_nns_node_provider_list_report();
            report.provenance.network.clone_from(&request.network);
            report.provenance.fetched_at.clone_from(&request.fetched_at);
            report
                .provenance
                .source_endpoint
                .clone_from(&request.endpoint);
            report.provenance.fetched_by.clone_from(&request.fetched_by);
            Ok(report)
        })
    }
}

#[cfg(feature = "nns-host")]
struct FixtureNnsNodeOperatorSource;

#[cfg(feature = "nns-host")]
impl NnsNodeOperatorSource for FixtureNnsNodeOperatorSource {
    fn fetch_node_operator_list_report<'a>(
        &'a self,
        request: &'a CollectionSourceRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<NnsNodeOperatorListReport, NnsInventoryError>>
    {
        Box::pin(async move {
            assert_inventory_source_request(
                &request.network,
                &request.endpoint,
                &request.fetched_at,
                &request.fetched_by,
            );
            let mut report = sample_nns_node_operator_list_report();
            report.provenance.network.clone_from(&request.network);
            report.provenance.fetched_at.clone_from(&request.fetched_at);
            report
                .provenance
                .source_endpoint
                .clone_from(&request.endpoint);
            report.provenance.fetched_by.clone_from(&request.fetched_by);
            Ok(report)
        })
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
    assert_ne!(fetched_at, "");
    assert_eq!(fetched_by, "ic-query");
}

#[test]
fn public_nns_proposal_api_is_constructible_and_renderable() {
    let mut request = NnsProposalListRequest::new(proposal_governance_request(1_700_000_000), 25);
    request.before_proposal_id = Some(132_500);
    request.status = NnsProposalStatusFilter::Executed;
    request.reward_status = NnsProposalRewardStatusFilter::Settled;
    request.topic = NnsProposalTopicFilter::Governance;
    request.proposer_neuron_id = Some(12_345);
    request.query = Some("subnet".into());
    request = request.with_sort(NnsProposalListSort::TallyTime);
    request.sort_direction = SortDirection::Desc;

    assert_eq!(request.status.as_str(), "executed");
    assert_eq!(request.reward_status.as_str(), "settled");
    assert_eq!(request.topic.as_str(), "governance");
    assert_eq!(request.sort.direction_label(request.sort_direction), "desc");

    let proposal = sample_nns_proposal_row();
    let list_report = NnsProposalListReport {
        provenance: ic_query::report::ReportProvenance::Cache {
            cache_path: "/cache/nns/ic/governance/proposals/full.json".to_string(),
        },
        context: governance_report_context(&request.governance),
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

        proposals: vec![proposal.clone()],
    };

    let list_json = serde_json::to_value(&list_report).expect("serialize NNS proposal list report");
    let list_text =
        nns_proposal_list_report_text(&list_report, ic_query_text::TextOptions { verbose: true });

    assert_eq!(list_json["proposals"][0]["topic"], 4);
    assert_eq!(list_json["proposals"][0]["status"], 4);
    assert_eq!(list_json["proposals"][0]["reward_status"], 3);
    assert_eq!(list_json["proposals"][0]["ballots"][0]["vote"], 1);
    assert!(list_text.contains("proposal_count: 1"));
    assert!(list_text.contains("topic_filter: governance"));
    assert!(!list_text.contains("proposal_details:"));
    assert!(list_text.contains("Upgrade subnet"));

    let detail_request =
        NnsProposalRequest::new(proposal_governance_request(1_700_000_000), 132_411);
    let detail_report = NnsProposalReport {
        provenance: ic_query::report::ReportProvenance::Live,
        context: governance_report_context(&detail_request.governance),
        proposal_id: detail_request.proposal_id,

        proposal,
    };

    let detail_text =
        nns_proposal_report_text(&detail_report, ic_query_text::TextOptions { verbose: true });

    assert!(detail_text.contains("proposal_id: 132411"));
    assert!(!detail_text.contains("show_ballots:"));
    assert_eq!(
        serde_json::to_value(&detail_report).unwrap()["proposal"]["reject_cost_e8s"],
        100_000_000
    );
    assert!(!detail_text.contains("ballots:"));
    assert!(detail_text.contains("Upgrade subnet"));
}

#[test]
fn public_nns_topology_summary_and_versions_api_is_constructible_and_renderable() {
    let request = NnsInventoryListRequest::new(
        ic_query::nns::NnsInventoryCacheRequest::new(".", "ic"),
        "https://icp-api.io",
        1_700_000_000,
    );
    assert_eq!(request.cache.network, "ic");

    let registry_version = sample_topology_registry_version_row();
    let summary = NnsTopologySummaryReport {
        schema_version: 1,
        network: request.cache.network,
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
        join_coverage: ic_query::nns::topology::NnsTopologyJoinCoverageCounts {
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
        },
        subnet_catalog_stale: false,
        subnet_catalog_stale_reason: "fresh".to_string(),
        registry_versions: vec![registry_version.clone()],
    };
    let summary_json = serde_json::to_value(&summary).expect("summary JSON");
    assert_eq!(summary_json["nodes_with_known_node_provider_count"], 3);
    assert_eq!(summary_json.get("join_coverage"), None);
    assert_eq!(
        serde_json::from_value::<NnsTopologySummaryReport>(summary_json).unwrap(),
        summary
    );
    let summary_text = nns_topology_summary_report_text(&summary);
    assert!(summary_text.contains("topology: ic subnets 2 nodes 3"));
    assert!(summary_text.contains("subnet_catalog"));

    let versions = NnsTopologyVersionsReport {
        schema_version: 1,
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
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
        checks: vec![NnsTopologyCheckRow {
            check: "registry_versions".to_string(),
            status: NnsTopologyAssessmentStatus::Ok,
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
        join_coverage: ic_query::nns::topology::NnsTopologyJoinCoverageCounts {
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
        },
    };
    let coverage_json = serde_json::to_value(&coverage).expect("coverage JSON");
    assert_eq!(
        coverage_json["node_operators_with_known_data_center_count"],
        1
    );
    assert_eq!(coverage_json.get("join_coverage"), None);
    assert_eq!(
        serde_json::from_value::<NnsTopologyCoverageReport>(coverage_json).unwrap(),
        coverage
    );
    assert!(nns_topology_coverage_report_text(&coverage).contains("nodes"));
}

#[test]
fn public_nns_topology_gaps_and_capacity_api_is_constructible_and_renderable() {
    let gaps = NnsTopologyGapsReport {
        schema_version: 1,
        network: "ic".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        status: NnsTopologyAssessmentStatus::Attention,
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

    let mut refresh_request = NnsTopologyRefreshRequest::new(NnsInventoryListRequest::new(
        ic_query::nns::NnsInventoryCacheRequest::new(".", "ic"),
        "https://icp-api.io",
        1_700_000_000,
    ));
    refresh_request.dry_run = true;

    let refresh = NnsTopologyRefreshReport {
        schema_version: 1,
        network: refresh_request.read.cache.network,
        source_endpoint: refresh_request.read.source_endpoint,
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
#[tokio::test(flavor = "current_thread")]
async fn public_nns_topology_host_api_accepts_custom_source_adapter() {
    let source = FixtureNnsTopologySource::default();
    let request = NnsInventoryListRequest::new(
        ic_query::nns::NnsInventoryCacheRequest::new(".", "ic"),
        DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT,
        1_700_000_000,
    );

    let summary = build_nns_topology_summary_report_with_source(&request, &source)
        .await
        .expect("topology summary report");
    let coverage = build_nns_topology_coverage_report_with_source(
        &NnsInventoryListRequest::new(
            ic_query::nns::NnsInventoryCacheRequest::new(".", "ic"),
            DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT,
            1_700_000_000,
        ),
        &source,
    )
    .await
    .expect("topology coverage report");
    let check = build_nns_topology_check_report_with_source(
        &NnsInventoryListRequest::new(
            ic_query::nns::NnsInventoryCacheRequest::new(".", "ic"),
            DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT,
            1_700_000_000,
        ),
        &source,
    )
    .await
    .expect("topology check report");
    let versions = topology_versions_report_with_source(&source).await;
    let refresh = topology_refresh_report_with_source(&source).await;

    assert_eq!(summary.network, "ic");
    assert_eq!(summary.subnet_count, 1);
    assert_eq!(coverage.node_count, 1);
    assert_eq!(check.network, "ic");
    assert_eq!(versions.registry_versions.len(), 5);
    assert_eq!(refresh.component_count, 5);
    assert!(refresh.dry_run);
    assert_topology_direct_reports_with_source(&source).await;
}

#[cfg(feature = "nns-host")]
#[tokio::test(flavor = "current_thread")]
async fn public_topology_refresh_rejects_mismatched_inventory_scope_before_next_source_call() {
    let mut request = NnsTopologyRefreshRequest::new(NnsInventoryListRequest::new(
        ic_query::nns::NnsInventoryCacheRequest::new(".", "ic"),
        DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT,
        1_700_000_000,
    ));
    request.dry_run = true;

    for collection in ["node_providers", ""] {
        let source = FixtureNnsTopologySource {
            node_refresh_collection: Some(collection),
        };
        let error = refresh_nns_topology_report_with_source(&request, &source)
            .await
            .expect_err("wrong inventory scope");
        assert!(
            matches!(error, NnsTopologyHostError::Inventory(NnsInventoryError::InvalidSourceData { reason, .. }) if reason.contains("collection"))
        );
    }
}

#[cfg(feature = "nns-host")]
async fn topology_versions_report_with_source(
    source: &dyn NnsTopologySource,
) -> NnsTopologyVersionsReport {
    build_nns_topology_versions_report_with_source(
        &NnsInventoryListRequest::new(
            ic_query::nns::NnsInventoryCacheRequest::new(".", "ic"),
            DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT,
            1_700_000_000,
        ),
        source,
    )
    .await
    .expect("topology versions report")
}

#[cfg(feature = "nns-host")]
async fn topology_refresh_report_with_source(
    source: &dyn NnsTopologyRefreshSource,
) -> NnsTopologyRefreshReport {
    refresh_nns_topology_report_with_source(
        &{
            let mut configured_request =
                NnsTopologyRefreshRequest::new(NnsInventoryListRequest::new(
                    ic_query::nns::NnsInventoryCacheRequest::new(".", "ic"),
                    DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT,
                    1_700_000_000,
                ));
            configured_request.dry_run = true;
            configured_request
        },
        source,
    )
    .await
    .expect("topology refresh report")
}

#[cfg(feature = "nns-host")]
async fn assert_topology_direct_reports_with_source(source: &dyn NnsTopologySource) {
    let gaps_request: NnsInventoryListRequest = topology_read_request();
    let capacity_request: NnsInventoryListRequest = topology_read_request();
    let regions_request: NnsInventoryListRequest = topology_read_request();
    let providers_request: NnsInventoryListRequest = topology_read_request();
    let gaps = build_nns_topology_gaps_report_with_source(&gaps_request, source)
        .await
        .expect("gaps report");
    let capacity = build_nns_topology_capacity_report_with_source(&capacity_request, source)
        .await
        .expect("capacity report");
    let regions = build_nns_topology_regions_report_with_source(&regions_request, source)
        .await
        .expect("regions report");
    let providers = build_nns_topology_providers_report_with_source(&providers_request, source)
        .await
        .expect("providers report");

    assert_eq!(gaps.network, "ic");
    assert_eq!(capacity.capacity.len(), 1);
    assert_eq!(regions.regions.len(), 1);
    assert_eq!(providers.registered_node_provider_count, 1);
}

#[cfg(feature = "nns-host")]
fn topology_read_request() -> NnsInventoryListRequest {
    NnsInventoryListRequest::new(
        ic_query::nns::NnsInventoryCacheRequest::new(".", "ic"),
        DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT,
        1_700_000_000,
    )
}

#[cfg(feature = "nns-host")]
#[derive(Default)]
struct FixtureNnsTopologySource {
    node_refresh_collection: Option<&'static str>,
}

#[cfg(feature = "nns-host")]
impl NnsTopologySource for FixtureNnsTopologySource {
    fn fetch_subnet_catalog_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<SubnetCatalogListReport, NnsTopologyHostError>>
    {
        Box::pin(async move {
            assert_topology_source_request(request);
            let mut report = sample_subnet_catalog_list_report();
            report.network.clone_from(&request.cache.network);
            report
                .metadata
                .fetched_at
                .clone_from(&request.source_request().fetched_at);
            Ok(report)
        })
    }

    fn fetch_node_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<NnsNodeListReport, NnsTopologyHostError>> {
        Box::pin(async move {
            assert_topology_source_request(request);
            let mut report = sample_nns_node_list_report();
            stamp_topology_component_report(
                request,
                &mut report.provenance.network,
                &mut report.provenance.source_endpoint,
            );
            report
                .provenance
                .fetched_at
                .clone_from(&request.source_request().fetched_at);
            report
                .provenance
                .fetched_by
                .clone_from(&request.source_request().fetched_by);
            Ok(report)
        })
    }

    fn fetch_node_provider_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<NnsNodeProviderListReport, NnsTopologyHostError>>
    {
        Box::pin(async move {
            assert_topology_source_request(request);
            let mut report = sample_nns_node_provider_list_report();
            stamp_topology_component_report(
                request,
                &mut report.provenance.network,
                &mut report.provenance.source_endpoint,
            );
            report
                .provenance
                .fetched_at
                .clone_from(&request.source_request().fetched_at);
            report
                .provenance
                .fetched_by
                .clone_from(&request.source_request().fetched_by);
            Ok(report)
        })
    }

    fn fetch_node_operator_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<NnsNodeOperatorListReport, NnsTopologyHostError>>
    {
        Box::pin(async move {
            assert_topology_source_request(request);
            let mut report = sample_nns_node_operator_list_report();
            stamp_topology_component_report(
                request,
                &mut report.provenance.network,
                &mut report.provenance.source_endpoint,
            );
            report
                .provenance
                .fetched_at
                .clone_from(&request.source_request().fetched_at);
            report
                .provenance
                .fetched_by
                .clone_from(&request.source_request().fetched_by);
            Ok(report)
        })
    }

    fn fetch_data_center_list_report<'a>(
        &'a self,
        request: &'a NnsInventoryListRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<NnsDataCenterListReport, NnsTopologyHostError>>
    {
        Box::pin(async move {
            assert_topology_source_request(request);
            let mut report = sample_nns_data_center_list_report();
            stamp_topology_component_report(
                request,
                &mut report.provenance.network,
                &mut report.provenance.source_endpoint,
            );
            report
                .provenance
                .fetched_at
                .clone_from(&request.source_request().fetched_at);
            report
                .provenance
                .fetched_by
                .clone_from(&request.source_request().fetched_by);
            Ok(report)
        })
    }
}

#[cfg(feature = "nns-host")]
impl NnsTopologyRefreshSource for FixtureNnsTopologySource {
    fn refresh_subnet_catalog_report<'a>(
        &'a self,
        request: &'a ic_query::nns::topology::NnsTopologyRefreshRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<SubnetCatalogRefreshReport, NnsTopologyHostError>>
    {
        Box::pin(async move {
            assert_topology_refresh_source_request(request);
            Ok(sample_subnet_catalog_refresh_report())
        })
    }

    fn refresh_node_report<'a>(
        &'a self,
        request: &'a ic_query::nns::topology::NnsTopologyRefreshRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<NnsInventoryRefreshReport, NnsTopologyHostError>>
    {
        Box::pin(async move {
            assert_topology_refresh_source_request(request);
            let mut report = sample_nns_node_refresh_report(&request.read.cache.cache_root);
            if let Some(collection) = self.node_refresh_collection {
                report.collection = collection.to_string();
            }
            Ok(report)
        })
    }

    fn refresh_node_provider_report<'a>(
        &'a self,
        request: &'a ic_query::nns::topology::NnsTopologyRefreshRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<NnsInventoryRefreshReport, NnsTopologyHostError>>
    {
        Box::pin(async move {
            assert_topology_refresh_source_request(request);
            assert!(
                self.node_refresh_collection.is_none(),
                "invalid node scope must stop further refresh calls"
            );
            Ok(sample_nns_node_provider_refresh_report(
                &request.read.cache.cache_root,
            ))
        })
    }

    fn refresh_node_operator_report<'a>(
        &'a self,
        request: &'a ic_query::nns::topology::NnsTopologyRefreshRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<NnsInventoryRefreshReport, NnsTopologyHostError>>
    {
        Box::pin(async move {
            assert_topology_refresh_source_request(request);
            Ok(sample_nns_node_operator_refresh_report(
                &request.read.cache.cache_root,
            ))
        })
    }

    fn refresh_data_center_report<'a>(
        &'a self,
        request: &'a ic_query::nns::topology::NnsTopologyRefreshRequest,
    ) -> ic_query::report::SourceFuture<'a, Result<NnsInventoryRefreshReport, NnsTopologyHostError>>
    {
        Box::pin(async move {
            assert_topology_refresh_source_request(request);
            Ok(sample_nns_data_center_refresh_report(
                &request.read.cache.cache_root,
            ))
        })
    }
}

#[cfg(feature = "nns-host")]
fn stamp_topology_component_report(
    request: &NnsInventoryListRequest,
    network: &mut String,
    source_endpoint: &mut String,
) {
    network.clone_from(&request.cache.network);
    source_endpoint.clone_from(&request.source_endpoint);
}

#[cfg(feature = "nns-host")]
fn assert_topology_source_request(request: &NnsInventoryListRequest) {
    assert_eq!(request.cache.cache_root, PathBuf::from("."));
    assert_eq!(request.cache.network, "ic");
    assert_eq!(
        request.source_endpoint,
        DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT
    );
    assert_eq!(request.now_unix_secs, 1_700_000_000);
    assert_ne!(request.source_request().fetched_at, "");
    assert_eq!(request.source_request().fetched_by, "ic-query");
}

#[cfg(feature = "nns-host")]
fn assert_topology_refresh_source_request(
    request: &ic_query::nns::topology::NnsTopologyRefreshRequest,
) {
    assert_eq!(request.read.cache.cache_root, PathBuf::from("."));
    assert_eq!(request.read.cache.network, "ic");
    assert_eq!(
        request.read.source_endpoint,
        DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT
    );
    assert_eq!(request.read.now_unix_secs, 1_700_000_000);

    assert!(request.dry_run);
    assert_ne!(request.read.source_request().fetched_at, "");
    assert_eq!(request.read.source_request().fetched_by, "ic-query");
}

#[cfg(feature = "nns-host")]
#[test]
fn public_nns_proposal_host_api_reads_complete_cache_without_cli() {
    let root = temp_root("nns-proposal-host-public-api");
    write_nns_proposal_fixture_cache(&root);

    let cache_list_request = NnsGovernanceCacheRequest::new(&root, "ic");
    let cache_status_request = NnsGovernanceCacheRequest::new(&root, "ic");
    let mut list_request =
        NnsProposalListRequest::new(proposal_governance_request(1_700_000_000), 25);
    list_request.status = NnsProposalStatusFilter::Executed;
    list_request.reward_status = NnsProposalRewardStatusFilter::Settled;
    list_request.topic = NnsProposalTopicFilter::Governance;
    list_request.proposer_neuron_id = Some(12_345);
    list_request.query = Some("subnet".into());
    list_request = list_request.with_sort(NnsProposalListSort::TallyTime);
    list_request.sort_direction = SortDirection::Desc;

    let detail_request =
        NnsProposalRequest::new(proposal_governance_request(1_700_000_000), 132_411);
    let mut refresh_request = NnsGovernanceRefreshRequest::new(
        &root,
        "ic",
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT,
        1_700_000_000,
        100,
    );
    refresh_request.max_pages = Some(1);

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
    assert_eq!(cache_list.caches.len(), 1);
    assert!(cache_status.found);
    assert_eq!(proposal_list.provenance.data_source().as_str(), "cache");
    assert_eq!(proposal_list.proposals.len(), 1);
    assert_eq!(cached_detail.provenance.data_source().as_str(), "cache");
    assert_eq!(cached_detail.proposal_id, 132_411);
    assert!(nns_proposal_cache_path(&root, "ic").is_file());
    assert!(nns_proposal_cache_root(&root, "ic").ends_with("proposals"));
    assert!(nns_proposal_refresh_lock_path(&root, "ic").ends_with("full.refresh.lock"));
    assert!(nns_proposal_refresh_attempt_path(&root, "ic").ends_with("full.refresh-attempt.json"));

    assert!(nns_proposal_cache_list_report_text(&cache_list).contains("cache_count: 1"));
    assert!(
        ic_query_text::nns::governance::nns_governance_cache_status_report_text(&cache_status)
            .contains("found: yes")
    );
    assert!(nns_governance_refresh_report_text(&refresh_report).contains("row_count: 1"));
    assert!(async_request_report_api_accepts_public_types(
        build_nns_proposal_list_report,
        &list_request
    ));
    assert!(async_request_report_api_accepts_public_types(
        build_nns_proposal_report,
        &detail_request
    ));
    assert!(async_request_report_api_accepts_public_types(
        refresh_nns_proposal_cache,
        &refresh_request
    ));

    let _ = fs::remove_dir_all(root);
}

#[cfg(feature = "nns-host")]
#[tokio::test(flavor = "current_thread")]
async fn public_nns_proposal_host_api_accepts_custom_source_adapter() {
    let root = temp_root("nns-proposal-source-public-api");
    let source = FixtureNnsProposalSource;
    let mut list_request =
        NnsProposalListRequest::new(proposal_governance_request(1_700_000_000), 25);
    list_request.status = NnsProposalStatusFilter::Executed;
    list_request.reward_status = NnsProposalRewardStatusFilter::Settled;
    list_request.topic = NnsProposalTopicFilter::Governance;

    let detail_request =
        NnsProposalRequest::new(proposal_governance_request(1_700_000_000), 132_411);
    let refresh_request = NnsGovernanceRefreshRequest::new(
        &root,
        "ic",
        DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT,
        1_700_000_000,
        2,
    );

    let list = build_nns_proposal_list_report_with_source(&list_request, &source)
        .await
        .expect("proposal list report");
    let detail = build_nns_proposal_report_with_source(&detail_request, &source)
        .await
        .expect("proposal detail report");
    let refresh =
        refresh_nns_proposal_cache_with_source_and_progress(&refresh_request, &source, &mut |_| {})
            .await
            .expect("proposal refresh report");

    assert_eq!(list.proposals.len(), 1);
    assert_eq!(list.provenance.data_source().as_str(), "live");
    assert_eq!(detail.proposal_id, 132_411);
    assert_eq!(detail.proposal.title.as_deref(), Some("Upgrade subnet"));
    assert_eq!(refresh.row_count, 1);
    assert!(refresh.complete);
    assert!(nns_proposal_cache_path(&root, "ic").is_file());

    let _ = fs::remove_dir_all(root);
}

struct FixtureNnsProposalSource;

struct PagedFixtureNnsProposalSource;

struct ChangingCanisterProposalSource;

impl NnsProposalSource for FixtureNnsProposalSource {
    fn fetch_proposals<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        status: NnsProposalStatusFilter,
        reward_status: NnsProposalRewardStatusFilter,
    ) -> NnsProposalSourceFuture<'a, Vec<NnsProposalRow>> {
        Box::pin(async move {
            assert_proposal_source_request(request);
            match (limit, before_proposal_id, status, reward_status) {
                (
                    25,
                    None,
                    NnsProposalStatusFilter::Executed,
                    NnsProposalRewardStatusFilter::Settled,
                )
                | (2, None, NnsProposalStatusFilter::Any, NnsProposalRewardStatusFilter::Any) => {
                    Ok(NnsGovernanceSourceData::new(
                        vec![sample_nns_proposal_row()],
                        governance_fixture_provenance(request),
                    ))
                }
                other => panic!("unexpected proposal source call: {other:?}"),
            }
        })
    }

    fn fetch_proposal<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        proposal_id: u64,
    ) -> NnsProposalSourceFuture<'a, NnsProposalRow> {
        Box::pin(async move {
            assert_proposal_source_request(request);
            assert_eq!(proposal_id, 132_411);
            Ok(NnsGovernanceSourceData::new(
                sample_nns_proposal_row(),
                governance_fixture_provenance(request),
            ))
        })
    }
}

impl NnsProposalSource for PagedFixtureNnsProposalSource {
    fn fetch_proposals<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        status: NnsProposalStatusFilter,
        reward_status: NnsProposalRewardStatusFilter,
    ) -> NnsProposalSourceFuture<'a, Vec<NnsProposalRow>> {
        Box::pin(async move {
            assert_proposal_source_request(request);
            assert_eq!(limit, 2);
            assert_eq!(status, NnsProposalStatusFilter::Any);
            assert_eq!(reward_status, NnsProposalRewardStatusFilter::Any);
            let proposals = match before_proposal_id {
                None => vec![
                    sample_nns_proposal_row_with_id(3),
                    sample_nns_proposal_row_with_id(2),
                ],
                Some(2) => vec![sample_nns_proposal_row_with_id(1)],
                other => panic!("unexpected proposal collection cursor: {other:?}"),
            };
            Ok(NnsGovernanceSourceData::new(
                proposals,
                governance_fixture_provenance(request),
            ))
        })
    }

    fn fetch_proposal<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
        _proposal_id: u64,
    ) -> NnsProposalSourceFuture<'a, NnsProposalRow> {
        panic!("proposal collection does not fetch exact detail")
    }
}

impl NnsProposalSource for ChangingCanisterProposalSource {
    fn fetch_proposals<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        status: NnsProposalStatusFilter,
        reward_status: NnsProposalRewardStatusFilter,
    ) -> NnsProposalSourceFuture<'a, Vec<NnsProposalRow>> {
        Box::pin(async move {
            assert_eq!(limit, 2);
            assert_eq!(status, NnsProposalStatusFilter::Any);
            assert_eq!(reward_status, NnsProposalRewardStatusFilter::Any);
            let (proposals, collector_canister_id) = match before_proposal_id {
                None => (
                    vec![
                        sample_nns_proposal_row_with_id(3),
                        sample_nns_proposal_row_with_id(2),
                    ],
                    "aaaaa-aa",
                ),
                Some(2) => (vec![sample_nns_proposal_row_with_id(1)], "2vxsx-fae"),
                other => panic!("unexpected proposal collection cursor: {other:?}"),
            };
            Ok(NnsGovernanceSourceData::new(
                proposals,
                NnsGovernanceSourceProvenance::ReplicatedInterCanisterCall {
                    collector_canister_id: collector_canister_id.to_string(),
                },
            ))
        })
    }

    fn fetch_proposal<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
        _proposal_id: u64,
    ) -> NnsProposalSourceFuture<'a, NnsProposalRow> {
        panic!("proposal collection does not fetch exact detail")
    }
}

fn sample_nns_proposal_row_with_id(proposal_id: u64) -> NnsProposalRow {
    NnsProposalRow {
        proposal_id: Some(proposal_id),
        ..sample_nns_proposal_row()
    }
}

fn assert_proposal_source_request(request: &NnsGovernanceRequest) {
    assert_eq!(request.network, "ic");
    assert_ne!(request.fetched_at, "");
    assert_eq!(
        request.source,
        NnsGovernanceSourceSelection::ReplicaQuery {
            endpoint: DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT.to_string(),
            fetched_by: "ic-query".to_string(),
        }
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
            "metadata": {"governance_canister_id": "rrkah-fqaaa-aaaaa-aaaaq-cai"},
            "completeness": {
                "status": "api_exhausted",
                "page_size": 100,
                "page_count": 1,
                "row_count": 1,
                "point_in_time_guaranteed": false
            },
            "data": { "proposals": [sample_nns_proposal_row()],
            "collection_evidence": {
                "requested_source": {"source_transport": "replica_query", "endpoint": DEFAULT_NNS_PROPOSAL_SOURCE_ENDPOINT, "fetched_by": "ic-query"},
                "max_pages": 1,
                "started_at": "2023-11-14T22:13:20Z"
            }}
        }),
    );
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
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("secure cache file");
    }
}

#[cfg(feature = "nns-host")]
#[must_use]
fn temp_root(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir()
        .canonicalize()
        .expect("physical temporary directory");
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
        catalog_schema_version: ic_query::subnet_catalog::CATALOG_SCHEMA_VERSION,
        metadata: ic_query::subnet_catalog::SubnetCatalogReportMetadata {
            registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
            registry_version: 42,
            assurance: CatalogAssurance::UncertifiedQuery,
            source_endpoints: vec![DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT.to_string()],
            agreement_digest: None,
            registry_query_call_count: 5,
            routing_source:
                ic_query::subnet_catalog::SubnetCatalogRoutingSource::LegacyRoutingTable,
            registry_records: Vec::new(),
            catalog_digest: "00".repeat(32),
            fetched_at: "2023-11-14T22:13:20Z".to_string(),
            resolver_backend: "local-nns-subnet-catalog".to_string(),
            collector_version: "test".to_string(),
            classification_schema_version: 1,
            classification_policy_digest: "00".repeat(32),
            resolver_schema_version: 1,
        },
        cache_disposition: CacheDisposition::CacheHit,
        catalog_stale: false,
        stale_reason: "fresh".to_string(),
        subnets: vec![sample_subnet_catalog_row()],
    }
}

#[cfg(feature = "nns-host")]
fn sample_subnet_catalog_refresh_report() -> SubnetCatalogRefreshReport {
    SubnetCatalogRefreshReport {
        schema_version: 1,
        network: "ic".to_string(),
        catalog_path: "/cache/nns/ic/subnet-catalog/catalog.json".to_string(),
        refresh_lock_path: "/cache/nns/ic/subnet-catalog/refresh.lock".to_string(),
        output_path: None,
        metadata: ic_query::subnet_catalog::SubnetCatalogReportMetadata {
            registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
            registry_version: 42,
            assurance: CatalogAssurance::UncertifiedQuery,
            source_endpoints: vec![DEFAULT_NNS_TOPOLOGY_SOURCE_ENDPOINT.to_string()],
            agreement_digest: None,
            registry_query_call_count: 5,
            routing_source:
                ic_query::subnet_catalog::SubnetCatalogRoutingSource::LegacyRoutingTable,
            registry_records: Vec::new(),
            catalog_digest: "00".repeat(32),
            fetched_at: "2023-11-14T22:13:20Z".to_string(),
            collector_version: "test".to_string(),
            classification_schema_version: 1,
            classification_policy_digest: "00".repeat(32),
            resolver_schema_version: 1,
            resolver_backend: "local-nns-subnet-catalog".to_string(),
        },
        fetched_by: "ic-query".to_string(),
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
        classification: ic_query::subnet_catalog::SubnetClassification {
            registry_subnet_type: 1,
            subnet_kind,
            subnet_kind_source: ClassificationSource::Registry,
            subnet_specialization: SubnetSpecialization::None,
            subnet_specialization_source: ClassificationSource::Computed,
            geographic_scope: GeographicScope::Global,
            geographic_scope_source: ClassificationSource::Computed,
            subnet_label: subnet_kind.as_str().to_string(),
            subnet_label_source: ClassificationSource::Computed,
        },
        subnet_principal: "tdb26-jop6g-7sc54-foywl".to_string(),
        node_count: Some(1),
        charges_apply_by_default: subnet_kind.charges_apply_by_default(),
        range_count: 1,
        ranges_shown: 0,
        range_offset: 0,
        range_limit: 1,
        ranges: Vec::new(),
    }
}

fn sample_nns_node_list_report() -> NnsNodeListReport {
    let node = sample_nns_node_row();
    NnsNodeListReport {
        schema_version: 1,
        provenance: ic_query::nns::registry::NnsRegistryInventoryProvenance {
            source_endpoint: "https://icp-api.io".to_string(),
            ..sample_registry_provenance()
        },
        nodes: vec![node],
    }
}

fn sample_nns_data_center_list_report() -> NnsDataCenterListReport {
    let data_center = sample_nns_data_center_row();
    NnsDataCenterListReport {
        schema_version: 1,
        provenance: ic_query::nns::registry::NnsRegistryInventoryProvenance {
            source_endpoint: "https://icp-api.io".to_string(),
            ..sample_registry_provenance()
        },
        data_centers: vec![data_center],
    }
}

fn sample_nns_node_provider_list_report() -> NnsNodeProviderListReport {
    let provider = sample_nns_node_provider_row();
    NnsNodeProviderListReport {
        schema_version: 1,
        provenance: ic_query::nns::registry::NnsRegistryInventoryProvenance {
            source_endpoint: "https://icp-api.io".to_string(),
            ..sample_registry_provenance()
        },
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        node_providers: vec![provider],
    }
}

fn sample_nns_node_operator_list_report() -> NnsNodeOperatorListReport {
    let operator = sample_nns_node_operator_row();
    NnsNodeOperatorListReport {
        schema_version: 1,
        provenance: ic_query::nns::registry::NnsRegistryInventoryProvenance {
            source_endpoint: "https://icp-api.io".to_string(),
            ..sample_registry_provenance()
        },
        node_operators: vec![operator],
    }
}

#[cfg(feature = "nns-host")]
fn sample_nns_inventory_refresh_report(
    collection: &str,
    cache_path: &Path,
    refresh_lock_path: &Path,
    source_endpoint: &str,
    governance_canister_id: Option<&str>,
) -> NnsInventoryRefreshReport {
    NnsInventoryRefreshReport {
        schema_version: 1,
        collection: collection.to_string(),
        provenance: ic_query::nns::registry::NnsRegistryInventoryProvenance {
            source_endpoint: source_endpoint.to_string(),
            ..sample_registry_provenance()
        },
        cache_path: cache_path.display().to_string(),
        refresh_lock_path: refresh_lock_path.display().to_string(),
        output_path: None,
        governance_canister_id: governance_canister_id.map(str::to_string),
        dry_run: true,
        wrote_cache: false,
        replaced_existing_cache: true,
        row_count: 1,
    }
}

#[cfg(feature = "nns-host")]
fn sample_nns_node_refresh_report(root: &Path) -> NnsInventoryRefreshReport {
    sample_nns_inventory_refresh_report(
        "nodes",
        &nns_node_cache_path(root, "ic"),
        &nns_node_refresh_lock_path(root, "ic"),
        DEFAULT_NNS_NODE_SOURCE_ENDPOINT,
        None,
    )
}

#[cfg(feature = "nns-host")]
fn sample_nns_data_center_refresh_report(root: &Path) -> NnsInventoryRefreshReport {
    sample_nns_inventory_refresh_report(
        "data_centers",
        &nns_data_center_cache_path(root, "ic"),
        &nns_data_center_refresh_lock_path(root, "ic"),
        DEFAULT_NNS_DATA_CENTER_SOURCE_ENDPOINT,
        None,
    )
}

#[cfg(feature = "nns-host")]
fn sample_nns_node_provider_refresh_report(root: &Path) -> NnsInventoryRefreshReport {
    sample_nns_inventory_refresh_report(
        "node_providers",
        &nns_node_provider_cache_path(root, "ic"),
        &nns_node_provider_refresh_lock_path(root, "ic"),
        DEFAULT_NNS_NODE_PROVIDER_SOURCE_ENDPOINT,
        Some("rrkah-fqaaa-aaaaa-aaaaq-cai"),
    )
}

#[cfg(feature = "nns-host")]
fn sample_nns_node_operator_refresh_report(root: &Path) -> NnsInventoryRefreshReport {
    sample_nns_inventory_refresh_report(
        "node_operators",
        &nns_node_operator_cache_path(root, "ic"),
        &nns_node_operator_refresh_lock_path(root, "ic"),
        DEFAULT_NNS_NODE_OPERATOR_SOURCE_ENDPOINT,
        None,
    )
}

#[cfg(feature = "nns-host")]
fn sample_nns_proposal_refresh_report(root: &Path) -> NnsGovernanceRefreshReport {
    NnsGovernanceRefreshReport {
        schema_version: 1,
        network: "ic".to_string(),
        governance_canister_id: "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string(),
        row_count: 1,
        collection: ic_query::nns::NnsGovernanceCollection::Proposals,
        point_in_time_guaranteed: false,
        page_size: 100,
        page_count: 1,
        complete: true,
        replaced_existing_cache: false,
        wrote_cache: true,
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

        status: 4,

        reward_status: 3,

        title: Some("Upgrade subnet".to_string()),
        summary: "Upgrade subnet replica version.".to_string(),
        url: "https://dashboard.internetcomputer.org/proposal/132411".to_string(),
        action_text: Some("execute-nns-function".to_string()),
        reject_cost_e8s: 100_000_000,
        proposal_timestamp_seconds: 1_700_000_000,
        deadline_timestamp_seconds: Some(1_700_086_400),
        decided_timestamp_seconds: 1_700_010_000,
        executed_timestamp_seconds: 1_700_020_000,
        failed_timestamp_seconds: 0,
        reward_event_round: 42,
        total_potential_voting_power: Some(1_000_000_000),
        latest_tally: Some(NnsProposalTally {
            timestamp_seconds: 1_700_010_000,
            yes: 900_000_000,
            no: 100_000_000,
            total: 1_000_000_000,
        }),

        ballots: vec![NnsProposalBallotRow {
            neuron_id: 12_345,
            vote: 1,

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

#[cfg(feature = "nns-host")]
fn async_request_report_api_accepts_public_types<Request, Report, Error>(
    _build: impl AsyncFn(&Request) -> Result<Report, Error>,
    request: &Request,
) -> bool {
    std::mem::size_of_val(request) > 0
}

#[cfg(feature = "nns-host")]
#[test]
fn public_inventory_lists_round_trip_flat_provenance_and_reject_unknown_fields() {
    fn check<Report>(report: Report)
    where
        Report: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let mut json = serde_json::to_value(&report).expect("serialize inventory");
        assert_eq!(json["network"], "ic");
        assert_eq!(json["registry_version"], 42);
        assert_eq!(json["fetched_at"], "2023-11-14T22:13:20Z");
        assert_eq!(
            serde_json::from_value::<Report>(json.clone()).unwrap(),
            report
        );
        json["unexpected"] = serde_json::json!(true);
        assert!(serde_json::from_value::<Report>(json).is_err());
    }
    check(sample_nns_node_list_report());
    check(sample_nns_data_center_list_report());
    check(sample_nns_node_provider_list_report());
    check(sample_nns_node_operator_list_report());
}

fn sample_registry_provenance() -> ic_query::nns::registry::NnsRegistryInventoryProvenance {
    ic_query::nns::registry::NnsRegistryInventoryProvenance {
        network: "ic".to_string(),
        registry_canister_id: "rwlgt-iiaaa-aaaaa-aaaaa-cai".to_string(),
        registry_version: 42,
        fetched_at: "2023-11-14T22:13:20Z".to_string(),
        source_endpoint: "https://icp-api.io".to_string(),
        fetched_by: "ic-query".to_string(),
    }
}
