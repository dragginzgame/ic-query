use super::{FixtureSnsListSource, GOVERNANCE_A};
use crate::sns::report::tests::*;

pub(in crate::sns::report::tests) struct FixtureSnsProposalSource;

impl SnsListSource for FixtureSnsProposalSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        FixtureSnsListSource.fetch_deployed_snses(request)
    }
}

impl SnsProposalSource for FixtureSnsProposalSource {
    fn fetch_sns_proposal(
        &self,
        _request: &SnsFetchRequest,
        sns: &MainnetSns,
        proposal_id: u64,
    ) -> Result<MainnetSnsProposal, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        assert_eq!(proposal_id, 42);
        Ok(MainnetSnsProposal {
            proposal: fixture_proposal_row(),
        })
    }
}

impl SnsProposalSource for NoLiveSnsProposalsSource {
    fn fetch_sns_proposal(
        &self,
        _request: &SnsFetchRequest,
        _sns: &MainnetSns,
        _proposal_id: u64,
    ) -> Result<MainnetSnsProposal, SnsHostError> {
        Err(no_live_error("fetch_sns_proposal"))
    }
}

pub(in crate::sns::report::tests) struct FixtureSnsProposalsSource;

impl SnsListSource for FixtureSnsProposalsSource {
    fn fetch_deployed_snses(
        &self,
        request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        FixtureSnsListSource.fetch_deployed_snses(request)
    }
}

impl SnsProposalsSource for FixtureSnsProposalsSource {
    fn fetch_sns_proposals(
        &self,
        _request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &[i32],
        topic: SnsProposalTopicFilter,
    ) -> Result<MainnetSnsProposals, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        assert_eq!(limit, 10);
        assert_eq!(before_proposal_id, Some(99));
        assert_eq!(include_status, &[1]);
        assert_eq!(topic, SnsProposalTopicFilter::Governance);
        Ok(MainnetSnsProposals {
            proposals: vec![fixture_proposal_row()],
        })
    }

    fn fetch_sns_proposal_page(
        &self,
        _request: &SnsFetchRequest,
        sns: &MainnetSns,
        limit: u32,
        before_proposal_id: Option<u64>,
    ) -> Result<MainnetSnsProposalPage, SnsHostError> {
        assert_eq!(sns.governance_canister_id, GOVERNANCE_A);
        assert_eq!(limit, 100);
        assert_eq!(before_proposal_id, None);
        Ok(MainnetSnsProposalPage {
            proposals: vec![fixture_proposal_row()],
            last_cursor: Some(42),
        })
    }
}

pub(in crate::sns::report::tests) struct NoLiveSnsProposalsSource;

impl SnsListSource for NoLiveSnsProposalsSource {
    fn fetch_deployed_snses(
        &self,
        _request: &SnsFetchRequest,
    ) -> Result<MainnetSnsList, SnsHostError> {
        Err(no_live_error("fetch_deployed_snses"))
    }
}

impl SnsProposalsSource for NoLiveSnsProposalsSource {
    fn fetch_sns_proposals(
        &self,
        _request: &SnsFetchRequest,
        _sns: &MainnetSns,
        _limit: u32,
        _before_proposal_id: Option<u64>,
        _include_status: &[i32],
        _topic: SnsProposalTopicFilter,
    ) -> Result<MainnetSnsProposals, SnsHostError> {
        Err(no_live_error("fetch_sns_proposals"))
    }

    fn fetch_sns_proposal_page(
        &self,
        _request: &SnsFetchRequest,
        _sns: &MainnetSns,
        _limit: u32,
        _before_proposal_id: Option<u64>,
    ) -> Result<MainnetSnsProposalPage, SnsHostError> {
        Err(no_live_error("fetch_sns_proposal_page"))
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
        proposal_id: Some(42),
        action_id: 1,
        action: "motion".to_string(),
        title: "Fixture proposal".to_string(),
        summary: "Fixture proposal summary".to_string(),
        url: Some("https://example.com/proposal".to_string()),
        decision_state: "open".to_string(),
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
            vote_text: "yes".to_string(),
            cast_timestamp_seconds: 1_780_272_050,
            cast_at: Some("2026-06-01T00:00:50Z".to_string()),
            voting_power: 100_000_000,
        }],
        payload_text_rendering: Some("Rendered payload".to_string()),
        proposer_neuron_id: Some("000102".to_string()),
    }
}
