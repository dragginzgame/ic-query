use super::*;
use crate::nns::governance::{NnsGovernanceError, validate_governance_time_interval};

struct UnexpectedSource;
impl NnsNeuronSource for UnexpectedSource {
    fn fetch_neuron_page<'a>(
        &'a self,
        _: &'a NnsGovernanceRequest,
        _: Option<u64>,
        _: u32,
    ) -> crate::nns::neuron::NnsNeuronSourceFuture<'a, crate::nns::neuron::NnsNeuronPage> {
        panic!("unexpected source call")
    }
    fn fetch_neuron<'a>(
        &'a self,
        _: &'a NnsGovernanceRequest,
        _: u64,
    ) -> crate::nns::neuron::NnsNeuronSourceFuture<'a, crate::nns::neuron::NnsNeuronRow> {
        panic!("unexpected source call")
    }
}

#[tokio::test(flavor = "current_thread")]
async fn collection_rejects_backward_continuations_and_restored_intervals() {
    let start = NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 100);
    let state = NnsNeuronCollectionState::new(&start, 2, 3).unwrap();
    let before = NnsGovernanceRequest::replicated_inter_canister_call_from_unix_secs("ic", 99);
    let mut future = std::pin::pin!(advance_nns_neuron_collection_with_source(
        &before,
        &state,
        &UnexpectedSource
    ));
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
    let result = std::future::Future::poll(future.as_mut(), &mut context);
    assert!(matches!(
        result,
        std::task::Poll::Ready(Err(NnsNeuronError::Governance(
            NnsGovernanceError::InvalidTimestampOrder { .. }
        )))
    ));
    let mut value = serde_json::to_value(&state).unwrap();
    value["updated_at"] = serde_json::json!(before.fetched_at);
    let restored = serde_json::from_value(value).unwrap();
    assert!(matches!(
        validate_collection_state(&restored),
        Err(NnsNeuronError::Governance(
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
    let valid_state = NnsNeuronCollectionState::new(&request, 2, 3).expect("valid state");
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
                Err(NnsNeuronError::InvalidCollectionState { .. })
            ),
            "invalid restored {field}"
        );
    }
    for timestamp in ["not a UTC timestamp", "+2023-11-14T22:13:20Z"] {
        request.fetched_at = timestamp.to_string();
        assert!(matches!(
            NnsNeuronCollectionState::new(&request, 2, 3),
            Err(NnsNeuronError::Governance(
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
                Err(NnsNeuronError::Governance(
                    NnsGovernanceError::InvalidTimestamp { value, .. }
                )) if value == timestamp
            ));
        }
    }
}
