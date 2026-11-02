use super::*;
use crate::report::CollectionSourceRequest;
use crate::sns::report::SnsProposalRow;

struct InvalidSnsProposalSource;

delegate_sns_discovery!(InvalidSnsProposalSource);

impl SnsProposalSource for InvalidSnsProposalSource {
    fn fetch_sns_proposal<'a>(
        &'a self,
        _request: &'a CollectionSourceRequest,
        _sns: &'a MainnetSns,
        _proposal_id: u64,
    ) -> crate::report::SourceFuture<'a, Result<SnsProposalRow, SnsHostError>> {
        Box::pin(async move {
            let mut proposal = fixture_proposal_row();
            proposal.proposal_id = 99;
            Ok(proposal)
        })
    }
}

struct InvalidSnsProposalsSource;

delegate_sns_discovery!(InvalidSnsProposalsSource);

impl SnsProposalsSource for InvalidSnsProposalsSource {
    fn fetch_sns_proposals<'a>(
        &'a self,
        _request: &'a CollectionSourceRequest,
        _sns: &'a MainnetSns,
        _limit: u32,
        _before_proposal_id: Option<u64>,
        _include_status: &'a [i32],
        _topic: SnsProposalTopicFilter,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsProposalRow>, SnsHostError>> {
        Box::pin(async move {
            let proposal = fixture_proposal_row();
            Ok(vec![proposal.clone(), proposal])
        })
    }

    fn fetch_sns_proposal_page<'a>(
        &'a self,
        _request: &'a CollectionSourceRequest,
        _sns: &'a MainnetSns,
        _limit: u32,
        _before_proposal_id: Option<u64>,
    ) -> crate::report::SourceFuture<'a, Result<Vec<SnsProposalRow>, SnsHostError>> {
        Box::pin(async move {
            let mut proposal = fixture_proposal_row();
            proposal.proposal_id = 99;
            Ok(vec![proposal.clone(), proposal])
        })
    }
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposal_builders_reject_invalid_custom_source_rows() {
    let detail_error =
        build_sns_proposal_report_with_source(&proposal_request("1"), &InvalidSnsProposalSource)
            .await
            .expect_err("invalid exact proposal");
    assert!(matches!(
        detail_error,
        SnsHostError::InvalidSourceData {
            capability: "SNS proposal",
            reason,
        } if reason.contains("returned proposal id")
    ));

    let list_error = build_sns_proposals_report_with_source_and_progress(
        &proposals_request("1"),
        &InvalidSnsProposalsSource,
        &mut |_| {},
    )
    .await
    .expect_err("invalid bounded proposals");
    assert!(matches!(
        list_error,
        SnsHostError::InvalidSourceData {
            capability: "SNS proposals",
            reason,
        } if reason.contains("duplicate proposal id")
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposal_refresh_rejects_invalid_custom_source_page() {
    let root = crate::test_support::temp_dir("ic-query-invalid-sns-proposal-page");
    let error = refresh_sns_proposals_cache_with_source(
        &sns_proposals_refresh_request(&root, None),
        &InvalidSnsProposalsSource,
    )
    .await
    .expect_err("invalid proposal page");

    assert!(matches!(
        error,
        SnsHostError::InvalidSourceData {
            capability: "SNS proposal page",
            reason,
        } if reason.contains("duplicate proposal id")
    ));
    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposal_resolves_list_id_and_renders_governance_proposal() {
    let request = proposal_request("1");

    let mut report = build_sns_proposal_report_with_source(&request, &FixtureSnsProposalSource)
        .await
        .expect("sns proposal report");

    assert_eq!(report.schema_version, SNS_PROPOSAL_REPORT_SCHEMA_VERSION);
    assert_eq!(report.context.instance.id, 1);
    assert_eq!(report.context.instance.name, "Fixture SNS");
    assert_eq!(report.context.instance.root_canister_id, ROOT_A);
    assert_eq!(report.context.governance_canister_id, GOVERNANCE_A);
    assert_eq!(report.proposal_id, 42);

    assert_eq!(report.proposal.proposal_id, 42);
    assert_eq!(report.proposal.action_id, 1);
    assert_eq!(
        report.proposal.decision_state(),
        SnsProposalDecisionState::Open
    );
    assert_eq!(report.proposal.ballot_count(), 1);
    assert_eq!(report.proposal.ballots[0].vote, 1);
    assert_eq!(report.provenance.data_source().as_str(), "live");
    assert_eq!(report.provenance.cache_path(), None);
    assert_eq!(report.provenance.cache_complete(), None);

    report.proposal.proposal_creation_timestamp_seconds = 0;
    report.proposal.decided_timestamp_seconds = None;
    report.proposal.executed_timestamp_seconds = Some(1_700_000_000);
    report.proposal.failed_timestamp_seconds = Some(0);
    report.proposal.ballots[0].cast_timestamp_seconds = 0;

    let json = serde_json::to_value(&report).expect("raw timestamp report");
    assert_eq!(json["proposal"]["proposal_creation_timestamp_seconds"], 0);
    assert_eq!(
        json["proposal"]["executed_timestamp_seconds"],
        1_700_000_000
    );
    assert_eq!(json["proposal"]["ballots"][0]["cast_timestamp_seconds"], 0);
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_resolves_list_id_and_renders_governance_proposals() {
    let request = proposals_request("1");

    let report = build_sns_proposals_report_with_source_and_progress(
        &request,
        &FixtureSnsProposalsSource,
        &mut |_| {},
    )
    .await
    .expect("sns proposals report");

    assert_eq!(report.schema_version, SNS_PROPOSALS_REPORT_SCHEMA_VERSION);
    assert_eq!(report.context.instance.id, 1);
    assert_eq!(report.context.instance.name, "Fixture SNS");
    assert_eq!(report.context.instance.root_canister_id, ROOT_A);
    assert_eq!(report.context.governance_canister_id, GOVERNANCE_A);
    assert_eq!(report.requested_limit, 10);
    assert_eq!(report.before_proposal_id, Some(99));
    assert_eq!(report.status_filter, "open");
    assert_eq!(report.topic_filter, "governance");
    assert_eq!(report.eligibility_filter, "any");
    assert_eq!(report.proposer_filter, None);
    assert_eq!(report.query_filter, None);
    assert_eq!(report.sort, "api");
    assert_eq!(report.sort_direction, "none");
    assert_eq!(report.proposal_count, 1);
    assert_eq!(report.proposals[0].proposal_id, 42);
    assert_eq!(report.proposals[0].action_id, 1);
    assert_eq!(
        report.proposals[0].decision_state(),
        SnsProposalDecisionState::Open
    );
    assert_eq!(report.proposals[0].reject_cost_e8s, 100_000_000);
    assert_eq!(
        report.proposals[0].proposal_creation_timestamp_seconds,
        1_780_272_000
    );
    assert_eq!(
        report.proposals[0]
            .latest_tally
            .as_ref()
            .map(|tally| tally.yes),
        Some(10)
    );
    assert_eq!(report.provenance.data_source().as_str(), "live");
    assert_eq!(report.provenance.cache_path(), None);
    assert_eq!(report.provenance.cache_complete(), None);
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_live_proposer_filter_applies_to_returned_rows() {
    let mut request = proposals_request("1");
    request.proposer_neuron_id = Some("000102".to_string());

    let report = build_sns_proposals_report_with_source_and_progress(
        &request,
        &FixtureSnsProposalsSource,
        &mut |_| {},
    )
    .await
    .expect("sns proposals report");

    assert_eq!(report.provenance.data_source().as_str(), "live");
    assert_eq!(report.proposer_filter.as_deref(), Some("000102"));
    assert_eq!(report.proposal_count, 1);
    assert_eq!(report.proposals[0].proposal_id, 42);
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_live_eligibility_filter_applies_to_returned_rows() {
    let mut request = proposals_request("1");
    request.eligibility = SnsProposalEligibilityFilter::No;

    let report = build_sns_proposals_report_with_source_and_progress(
        &request,
        &FixtureSnsProposalsSource,
        &mut |_| {},
    )
    .await
    .expect("sns proposals report");

    assert_eq!(report.provenance.data_source().as_str(), "live");
    assert_eq!(report.eligibility_filter, "no");
    assert_eq!(report.proposal_count, 0);
    assert_eq!(report.proposals, Vec::new());
}

#[tokio::test(flavor = "current_thread")]
async fn sns_proposals_live_query_filter_applies_to_returned_rows() {
    let mut request = proposals_request("1");
    request.query = Some("FiXtUrE PrOpOsAl".to_string());

    let report = build_sns_proposals_report_with_source_and_progress(
        &request,
        &FixtureSnsProposalsSource,
        &mut |_| {},
    )
    .await
    .expect("sns proposals report");

    let json = serde_json::to_value(&report).expect("serialize live SNS proposals report");

    assert_eq!(report.provenance.data_source().as_str(), "live");
    assert_eq!(report.query_filter.as_deref(), Some("FiXtUrE PrOpOsAl"));
    assert_eq!(report.proposal_count, 1);
    assert_eq!(report.proposals[0].proposal_id, 42);
    assert_eq!(json["query_filter"], "FiXtUrE PrOpOsAl");
}

#[test]
fn sns_proposals_status_decided_requires_cache_root() {
    let error = proposals_request("1")
        .with_status(SnsProposalStatusFilter::Decided)
        .unwrap_err();
    assert_eq!(error, crate::sns::SnsProposalViewError);
}
