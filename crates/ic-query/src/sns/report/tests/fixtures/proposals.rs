use super::{FixtureSnsDiscoverySource, GOVERNANCE_A};
use crate::sns::report::tests::*;

///
/// FixtureSnsProposalSource
///
/// Successful single-proposal SNS source used by report tests.
///

pub(in crate::sns::report::tests) struct FixtureSnsProposalSource;

delegate_sns_discovery!(FixtureSnsProposalSource);

impl SnsProposalSource for FixtureSnsProposalSource {
    fn fetch_sns_proposal<'a>(
        &'a self,
        _request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
        proposal_id: u64,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsProposal, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
            assert_eq!(proposal_id, 42);
            Ok(MainnetSnsProposal {
                proposal: fixture_proposal_row(),
            })
        })
    }
}

impl SnsProposalSource for NoLiveSnsProposalsSource {
    fn fetch_sns_proposal<'a>(
        &'a self,
        _request: &'a SnsSourceRequest,
        _sns: &'a MainnetSns,
        _proposal_id: u64,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsProposal, SnsHostError>> {
        Box::pin(async move { Err(no_live_error("fetch_sns_proposal")) })
    }
}

///
/// FixtureSnsProposalsSource
///
/// Successful bounded and paged SNS proposal source used by report tests.
///

pub(in crate::sns::report::tests) struct FixtureSnsProposalsSource;

delegate_sns_discovery!(FixtureSnsProposalsSource);

impl SnsProposalsSource for FixtureSnsProposalsSource {
    fn fetch_sns_proposals<'a>(
        &'a self,
        _request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &'a [i32],
        topic: SnsProposalTopicFilter,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsProposals, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
            assert_eq!(limit, 10);
            assert_eq!(before_proposal_id, Some(99));
            assert_eq!(include_status, &[1]);
            assert_eq!(topic, SnsProposalTopicFilter::Governance);
            Ok(MainnetSnsProposals {
                proposals: vec![fixture_proposal_row()],
            })
        })
    }

    fn fetch_sns_proposal_page<'a>(
        &'a self,
        _request: &'a SnsSourceRequest,
        sns: &'a MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsProposalPage, SnsHostError>> {
        Box::pin(async move {
            assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
            assert_eq!(limit, 100);
            assert_eq!(before_proposal_id, None);
            Ok(MainnetSnsProposalPage {
                proposals: vec![fixture_proposal_row()],
            })
        })
    }
}

///
/// NoLiveSnsProposalsSource
///
/// SNS proposal source that rejects live calls in cache-backed report tests.
///

pub(in crate::sns::report::tests) struct NoLiveSnsProposalsSource;

impl SnsDiscoverySource for NoLiveSnsProposalsSource {
    fn fetch_sns_inventory<'a>(
        &'a self,
        _request: &'a SnsSourceRequest,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsInventory, SnsHostError>> {
        Box::pin(async move { Err(no_live_error("fetch_sns_inventory")) })
    }

    fn fetch_sns_metadata<'a>(
        &'a self,
        _request: &'a SnsSourceRequest,
        _targets: &'a [MainnetSnsCanisters],
    ) -> crate::report::SourceFuture<'a, Result<Vec<MainnetSnsMetadata>, SnsHostError>> {
        Box::pin(async move { Err(no_live_error("fetch_sns_metadata")) })
    }
}

impl SnsProposalsSource for NoLiveSnsProposalsSource {
    fn fetch_sns_proposals<'a>(
        &'a self,
        _request: &'a SnsSourceRequest,
        _sns: &'a MainnetSns,
        _limit: u32,
        _before_proposal_id: Option<u64>,
        _include_status: &'a [i32],
        _topic: SnsProposalTopicFilter,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsProposals, SnsHostError>> {
        Box::pin(async move { Err(no_live_error("fetch_sns_proposals")) })
    }

    fn fetch_sns_proposal_page<'a>(
        &'a self,
        _request: &'a SnsSourceRequest,
        _sns: &'a MainnetSns,
        _limit: u32,
        _before_proposal_id: Option<u64>,
    ) -> crate::report::SourceFuture<'a, Result<MainnetSnsProposalPage, SnsHostError>> {
        Box::pin(async move { Err(no_live_error("fetch_sns_proposal_page")) })
    }
}

fn no_live_error(method: &'static str) -> SnsHostError {
    SnsHostError::AgentCall {
        method,
        reason: "unexpected live proposal call".to_string(),
    }
}

pub(in crate::sns::report::tests) fn fixture_proposal_row() -> SnsProposalRow {
    SnsProposalRow {
        proposal_id: 42,
        action_id: 1,
        action: SnsProposalAction::Motion,
        title: "Fixture proposal".to_string(),
        summary: "Fixture proposal summary".to_string(),
        url: Some("https://example.com/proposal".to_string()),
        decision_state: SnsProposalDecisionState::Open,
        status: Some(SNS_PROPOSAL_STATUS_OPEN_CODE),
        topic: Some(SnsProposalTopicFilter::Governance.as_str().to_string()),
        reject_cost_e8s: 100_000_000,
        proposal_creation_timestamp_seconds: 1_780_272_000,
        created_at: "2026-06-01T00:00:00Z".to_string(),
        decided_timestamp_seconds: None,
        decided_at: None,
        executed_timestamp_seconds: None,
        executed_at: None,
        failed_timestamp_seconds: None,
        failed_at: None,
        failure_reason: None,
        reward_event_round: 0,
        reward_event_end_timestamp_seconds: None,
        is_eligible_for_rewards: true,
        latest_tally: Some(SnsProposalTally {
            timestamp_seconds: 1_780_272_100,
            yes: 10,
            no: 2,
            total: 20,
        }),
        ballot_count: 1,
        ballots: vec![SnsProposalBallotRow {
            neuron_id: "000102030405".to_string(),
            vote: 1,
            vote_text: SnsProposalVote::Yes,
            cast_timestamp_seconds: 1_780_272_050,
            cast_at: Some("2026-06-01T00:00:50Z".to_string()),
            voting_power: 100_000_000,
        }],
        payload_text_rendering: Some("Rendered payload".to_string()),
        proposer_neuron_id: Some("000102".to_string()),
    }
}
