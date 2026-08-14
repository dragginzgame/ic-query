use super::*;

#[derive(Clone)]
struct StaticProposalSource {
    proposals: Vec<NnsProposalRow>,
    proposal: NnsProposalRow,
    provenance: Option<NnsGovernanceSourceProvenance>,
}

impl NnsProposalSource for StaticProposalSource {
    fn fetch_proposals<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        _limit: u32,
        _before_proposal_id: Option<u64>,
        _status: NnsProposalStatusFilter,
        _reward_status: NnsProposalRewardStatusFilter,
    ) -> NnsProposalSourceFuture<'a, Vec<NnsProposalRow>> {
        Box::pin(async move {
            Ok(NnsGovernanceSourceData::new(
                self.proposals.clone(),
                self.provenance
                    .clone()
                    .unwrap_or_else(|| fixture_provenance(request)),
            ))
        })
    }

    fn fetch_proposal<'a>(
        &'a self,
        request: &'a NnsGovernanceRequest,
        _proposal_id: u64,
    ) -> NnsProposalSourceFuture<'a, NnsProposalRow> {
        Box::pin(async move {
            Ok(NnsGovernanceSourceData::new(
                self.proposal.clone(),
                self.provenance
                    .clone()
                    .unwrap_or_else(|| fixture_provenance(request)),
            ))
        })
    }
}

struct PanicProposalSource;

impl NnsProposalSource for PanicProposalSource {
    fn fetch_proposals<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
        _limit: u32,
        _before_proposal_id: Option<u64>,
        _status: NnsProposalStatusFilter,
        _reward_status: NnsProposalRewardStatusFilter,
    ) -> NnsProposalSourceFuture<'a, Vec<NnsProposalRow>> {
        panic!("invalid list request must fail before source invocation")
    }

    fn fetch_proposal<'a>(
        &'a self,
        _request: &'a NnsGovernanceRequest,
        _proposal_id: u64,
    ) -> NnsProposalSourceFuture<'a, NnsProposalRow> {
        panic!("proposal source must not be called")
    }
}

#[test]
fn proposal_list_rejects_unbounded_limit_before_source_invocation() {
    let mut request = proposal_sort_request(NnsProposalListSort::Api);
    request.limit = NNS_PROPOSAL_MAX_PAGE_SIZE + 1;

    let error = build_nns_proposal_list_report_with_source(&request, &PanicProposalSource)
        .expect_err("oversized proposal page limit");

    assert!(matches!(
        error,
        NnsProposalError::InvalidLimit {
            limit: 101,
            maximum: 100
        }
    ));
}

#[test]
fn proposal_list_rejects_oversized_duplicate_and_out_of_cursor_pages() {
    let mut request = proposal_sort_request(NnsProposalListSort::Api);
    request.limit = 1;
    let source = static_source(
        vec![proposal_row(101), proposal_row(102)],
        proposal_row(101),
    );
    assert!(matches!(
        build_nns_proposal_list_report_with_source(&request, &source),
        Err(NnsProposalError::PageTooLarge {
            actual: 2,
            requested: 1
        })
    ));

    request.limit = 50;
    let source = static_source(vec![proposal_row(0)], proposal_row(101));
    assert!(matches!(
        build_nns_proposal_list_report_with_source(&request, &source),
        Err(NnsProposalError::InvalidProposalIdInPage)
    ));

    let source = static_source(
        vec![proposal_row(101), proposal_row(101)],
        proposal_row(101),
    );
    assert!(matches!(
        build_nns_proposal_list_report_with_source(&request, &source),
        Err(NnsProposalError::DuplicateProposalId { proposal_id: 101 })
    ));

    let source = static_source(vec![proposal_row(200)], proposal_row(101));
    assert!(matches!(
        build_nns_proposal_list_report_with_source(&request, &source),
        Err(NnsProposalError::ProposalCursorMismatch {
            proposal_id: 200,
            before_proposal_id: 200
        })
    ));
}

#[test]
fn proposal_detail_requires_the_exact_requested_id() {
    let request = NnsProposalRequest::new(proposal_governance_request(1_700_000_000), 101);
    let source = static_source(Vec::new(), proposal_row(102));

    let error = build_nns_proposal_report_with_source(&request, &source)
        .expect_err("mismatched proposal detail");

    assert!(matches!(
        error,
        NnsProposalError::ProposalIdMismatch {
            expected: 101,
            actual: Some(102)
        }
    ));
}

#[test]
fn proposal_builder_rejects_source_provenance_that_does_not_match_the_request() {
    let request = proposal_sort_request(NnsProposalListSort::Api);
    let source = StaticProposalSource {
        proposals: vec![proposal_row(101)],
        proposal: proposal_row(101),
        provenance: Some(NnsGovernanceSourceProvenance::ReplicatedInterCanisterCall {
            collector_canister_id: "aaaaa-aa".to_string(),
        }),
    };

    let error = build_nns_proposal_list_report_with_source(&request, &source)
        .expect_err("source provenance mismatch");

    assert!(matches!(
        error,
        NnsProposalError::Governance(NnsGovernanceError::SourceEvidenceMismatch { .. })
    ));
}

const fn static_source(
    proposals: Vec<NnsProposalRow>,
    proposal: NnsProposalRow,
) -> StaticProposalSource {
    StaticProposalSource {
        proposals,
        proposal,
        provenance: None,
    }
}

fn proposal_row(proposal_id: u64) -> NnsProposalRow {
    nns_proposal_row_from_info(proposal_info(
        proposal_id,
        NnsProposalTopic::Governance.code(),
        NnsProposalStatus::Executed.code(),
        "Portable proposal",
        20,
    ))
}
