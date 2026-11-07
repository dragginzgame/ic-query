use super::*;
use crate::nns::governance::NnsGovernanceSourceProvenance;
use crate::nns::governance::{NnsGovernanceError, validate_governance_time_interval};

#[test]
fn restored_collection_preserves_full_terminal_pages_and_cursor_validation_order() {
    let request = NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 100);
    let initial = NnsProposalCollectionState::new(&request, 2, 3).unwrap();
    let mut state = initial;
    state.collection.pages_fetched = 1;
    state.collection.source = Some(NnsGovernanceSourceProvenance::ReplicatedInterCanisterCall {
        collector_canister_id: "aaaaa-aa".to_string(),
    });
    state.proposals_fetched = 2;
    assert!(validate_collection_state(&state).is_ok());

    state.next_before_proposal_id = Some(1);
    assert!(matches!(
        validate_collection_state(&state),
        Err(NnsProposalError::InvalidCollectionState { reason })
            if reason.contains("next_before_proposal_id")
    ));
    state.proposals_fetched = 3;
    assert!(matches!(
        validate_collection_state(&state),
        Err(NnsProposalError::InvalidCollectionState { reason })
            if reason.contains("proposals_fetched")
    ));
}

struct UnexpectedSource;
impl NnsProposalSource for UnexpectedSource {
    fn fetch_proposals<'a>(
        &'a self,
        _: &'a NnsGovernanceRequest,
        _: u32,
        _: Option<u64>,
        _: crate::nns::proposals::NnsProposalStatusFilter,
        _: crate::nns::proposals::NnsProposalRewardStatusFilter,
    ) -> crate::nns::proposals::NnsProposalSourceFuture<
        'a,
        Vec<crate::nns::proposals::NnsProposalRow>,
    > {
        panic!("unexpected source call")
    }
    fn fetch_proposal<'a>(
        &'a self,
        _: &'a NnsGovernanceRequest,
        _: u64,
    ) -> crate::nns::proposals::NnsProposalSourceFuture<'a, crate::nns::proposals::NnsProposalRow>
    {
        panic!("unexpected source call")
    }
}

#[tokio::test(flavor = "current_thread")]
async fn collection_rejects_backward_continuations_and_restored_intervals() {
    let start = NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 100);
    let state = NnsProposalCollectionState::new(&start, 2, 3).unwrap();
    let before = NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 99);
    let mut future = std::pin::pin!(advance_nns_proposal_collection_with_source(
        &before,
        &state,
        &UnexpectedSource
    ));
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
    let result = std::future::Future::poll(future.as_mut(), &mut context);
    assert!(matches!(
        result,
        std::task::Poll::Ready(Err(NnsProposalError::Governance(
            NnsGovernanceError::InvalidTimestampOrder { .. }
        )))
    ));
    let mut value = serde_json::to_value(&state).unwrap();
    value["updated_at"] = serde_json::json!(before.fetched_at);
    let restored = serde_json::from_value(value).unwrap();
    assert!(matches!(
        validate_collection_state(&restored),
        Err(NnsProposalError::Governance(
            NnsGovernanceError::InvalidTimestampOrder { .. }
        ))
    ));
    assert!(
        validate_governance_time_interval(
            "updated_at",
            &state.collection.updated_at,
            "fetched_at",
            &start.fetched_at
        )
        .is_ok()
    );
}

#[test]
fn collection_validates_request_and_restored_timestamps() {
    let mut request =
        NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 1_700_000_000);
    let valid_state = NnsProposalCollectionState::new(&request, 2, 3).expect("valid state");
    for (field, value) in [
        ("governance_canister_id", serde_json::json!("aaaaa-aa")),
        ("max_pages", serde_json::json!(0)),
        ("pages_fetched", serde_json::json!(4)),
    ] {
        let mut restored = serde_json::to_value(&valid_state).unwrap();
        restored[field] = value;
        let restored = serde_json::from_value(restored).unwrap();
        assert!(
            matches!(
                validate_collection_state(&restored),
                Err(NnsProposalError::InvalidCollectionState { .. })
            ),
            "invalid restored {field}"
        );
    }
    for timestamp in ["not a UTC timestamp", "+2023-11-14T22:13:20Z"] {
        request.fetched_at = timestamp.to_string();
        assert!(matches!(
            NnsProposalCollectionState::new(&request, 2, 3),
            Err(NnsProposalError::Governance(
                NnsGovernanceError::InvalidTimestamp {
                    field: "fetched_at",
                    ..
                }
            ))
        ));
        for field in ["started_at", "updated_at"] {
            let mut value = serde_json::to_value(&valid_state).expect("serialize state");
            value[field] = serde_json::json!(timestamp);
            let state = serde_json::from_value(value).expect("restore caller state");
            assert!(matches!(
                validate_collection_state(&state),
                Err(NnsProposalError::Governance(
                    NnsGovernanceError::InvalidTimestamp { value, .. }
                )) if value == timestamp
            ));
        }
    }
}
