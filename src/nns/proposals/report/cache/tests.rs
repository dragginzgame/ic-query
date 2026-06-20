use super::{
    model::{
        NnsProposalCacheListRequest, NnsProposalCacheStatusRequest, NnsProposalRefreshRequest,
    },
    refresh::refresh_nns_proposal_cache_with_source,
    reports::{build_nns_proposal_cache_list_report, build_nns_proposal_cache_status_report},
};
use crate::{
    ic_registry::{DEFAULT_MAINNET_ENDPOINT, MAINNET_GOVERNANCE_CANISTER_ID},
    nns::proposals::report::{
        NnsProposalHostError,
        cache::paths::nns_proposal_cache_paths,
        source::{NnsProposalFetchRequest, NnsProposalSource},
        text::{nns_proposal_cache_status_report_text, nns_proposal_refresh_report_text},
        wire::{
            NnsGovernanceBallot, NnsNeuronId, NnsProposal, NnsProposalAction, NnsProposalId,
            NnsProposalInfo, NnsProposalTallyWire,
        },
    },
    subnet_catalog::MAINNET_NETWORK,
    test_support::temp_dir,
};
use candid::Reserved;

struct FixtureSource;

impl NnsProposalSource for FixtureSource {
    fn fetch_proposals(
        &self,
        _request: &NnsProposalFetchRequest,
        limit: u32,
        before_proposal_id: Option<u64>,
        include_status: &[i32],
        include_reward_status: &[i32],
    ) -> Result<Vec<NnsProposalInfo>, NnsProposalHostError> {
        assert_eq!(limit, 2);
        assert!(include_status.is_empty());
        assert!(include_reward_status.is_empty());
        Ok(match before_proposal_id {
            None => vec![proposal_info(3), proposal_info(2)],
            Some(2) => vec![proposal_info(1)],
            other => panic!("unexpected before proposal id: {other:?}"),
        })
    }

    fn fetch_proposal(
        &self,
        _request: &NnsProposalFetchRequest,
        proposal_id: u64,
    ) -> Result<NnsProposalInfo, NnsProposalHostError> {
        Ok(proposal_info(proposal_id))
    }
}

#[test]
fn nns_proposal_refresh_writes_complete_cache_and_status_reports() {
    let root = temp_dir("ic-query-nns-proposal-cache");
    let request = NnsProposalRefreshRequest {
        network: MAINNET_NETWORK.to_string(),
        source_endpoint: DEFAULT_MAINNET_ENDPOINT.to_string(),
        now_unix_secs: 1_700_000_000,
        icp_root: root.clone(),
        page_size: 2,
        max_pages: None,
    };

    let report =
        refresh_nns_proposal_cache_with_source(&request, &FixtureSource).expect("refresh cache");
    let refresh_text = nns_proposal_refresh_report_text(&report);

    assert_eq!(report.network, MAINNET_NETWORK);
    assert_eq!(
        report.governance_canister_id,
        MAINNET_GOVERNANCE_CANISTER_ID
    );
    assert_eq!(report.proposal_count, 3);
    assert_eq!(report.page_count, 2);
    assert!(report.complete);
    assert!(report.wrote_cache);
    assert!(!report.replaced_existing_cache);
    assert!(refresh_text.contains("proposal_count: 3"));
    assert!(
        nns_proposal_cache_paths(&root, MAINNET_NETWORK)
            .snapshot_path
            .is_file()
    );

    let list = build_nns_proposal_cache_list_report(&NnsProposalCacheListRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root.clone(),
    })
    .expect("cache list");

    assert_eq!(list.cache_count, 1);
    assert_eq!(list.caches[0].row_count, 3);
    assert_eq!(list.caches[0].page_count, 2);

    let status = build_nns_proposal_cache_status_report(&NnsProposalCacheStatusRequest {
        network: MAINNET_NETWORK.to_string(),
        icp_root: root,
    })
    .expect("cache status");
    let status_text = nns_proposal_cache_status_report_text(&status);

    assert!(status.found);
    assert_eq!(
        status
            .latest_attempt
            .as_ref()
            .expect("latest attempt")
            .status,
        "complete"
    );
    assert!(status_text.contains("latest_attempt:"));
    assert!(status_text.contains("status: complete"));
}

fn proposal_info(proposal_id: u64) -> NnsProposalInfo {
    NnsProposalInfo {
        id: Some(NnsProposalId { id: proposal_id }),
        status: 4,
        topic: 4,
        ballots: vec![(
            proposal_id,
            NnsGovernanceBallot {
                vote: 1,
                voting_power: 100,
            },
        )],
        proposal_timestamp_seconds: 1_700_000_000 + proposal_id,
        reward_event_round: 7,
        deadline_timestamp_seconds: Some(1_700_010_000),
        failed_timestamp_seconds: 0,
        reject_cost_e8s: 100_000_000,
        latest_tally: Some(NnsProposalTallyWire {
            no: 1,
            yes: 2,
            total: 3,
            timestamp_seconds: 1_700_000_100,
        }),
        reward_status: 3,
        decided_timestamp_seconds: 1_700_000_200,
        proposal: Some(NnsProposal {
            url: format!("https://dashboard.internetcomputer.org/proposal/{proposal_id}"),
            title: Some(format!("Proposal {proposal_id}")),
            action: Some(NnsProposalAction::Motion(Reserved)),
            summary: "Proposal summary".to_string(),
        }),
        proposer: Some(NnsNeuronId { id: 99 }),
        executed_timestamp_seconds: 1_700_000_300,
        total_potential_voting_power: Some(100),
    }
}
