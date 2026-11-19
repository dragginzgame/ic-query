use super::*;
use std::{cell::Cell, future::pending};

fn id(n: u8) -> Principal {
    Principal::from_slice(&[n, 1])
}

#[tokio::test]
async fn selection_admission_precedes_network() {
    assert!(matches!(
        fetch(&[]).await,
        Err(CollectionError::SelectionSize { count: 0 })
    ));
    assert!(matches!(
        fetch(&vec![id(1); 33]).await,
        Err(CollectionError::SelectionSize { count: 33 })
    ));
    assert!(matches!(
        fetch(&[id(1), id(1)]).await,
        Err(CollectionError::Duplicate { .. })
    ));
}

#[tokio::test]
async fn maximum_selection_preserves_order_and_independent_evidence() {
    let ids: Vec<_> = (0..32).rev().map(id).collect();
    validate(&ids).unwrap();
    let result = collect(&ids, 10, |selected| async move {
        Ok(super::super::report(selected, id(100), 20))
    })
    .await;
    assert!(result.collection_complete);
    assert!(!result.atomic_snapshot);
    assert!(!result.overall_deadline_reached);
    assert_eq!(
        result
            .canisters
            .iter()
            .map(|r| r.canister_id)
            .collect::<Vec<_>>(),
        ids
    );
    let json = serde_json::to_value(result).unwrap();
    assert_eq!(
        json["canisters"][0]["outcome"]["report"]["evidence"]["result_certified"],
        false
    );
    assert!(json["canisters"][0]["outcome"]["report"]["evidence"]["registry_version"].is_null());
}

#[tokio::test]
async fn failure_and_identity_mismatch_do_not_erase_later_success() {
    let result = collect(&[id(1), id(2), id(3)], 10, |selected| async move {
        if selected == id(1) {
            Err(PlacementError::RegistryRejected("raw\nreason".into()))
        } else if selected == id(2) {
            Ok(super::super::report(id(99), id(100), 20))
        } else {
            Ok(super::super::report(selected, id(100), 21))
        }
    })
    .await;
    assert!(!result.collection_complete);
    assert!(!result.overall_deadline_reached);
    assert!(
        matches!(result.canisters[1].outcome, PlacementOutcome::IdentityMismatch { observed_canister_id } if observed_canister_id == id(99))
    );
    assert!(matches!(
        result.canisters[2].outcome,
        PlacementOutcome::Read { .. }
    ));
    let json = serde_json::to_value(result).unwrap();
    assert_eq!(
        json["canisters"][0]["outcome"]["error"]["kind"],
        "registry_rejected"
    );
    assert_eq!(
        json["canisters"][0]["outcome"]["error"]["reason"],
        "raw\nreason"
    );
}

struct Guard<'a>(&'a Cell<usize>);
impl Drop for Guard<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[tokio::test(start_paused = true)]
async fn deadline_retains_completed_rows_cancels_active_and_never_starts_tail() {
    let calls = Cell::new(0);
    let dropped = Cell::new(0);
    let result = collect(&[id(1), id(2), id(3)], 10, |selected| {
        calls.set(calls.get() + 1);
        let dropped = &dropped;

        async move {
            if selected == id(1) {
                return Ok(super::super::report(selected, id(100), 20));
            }
            let _guard = Guard(dropped);

            pending().await
        }
    })
    .await;
    assert_eq!(calls.get(), 2);
    assert_eq!(dropped.get(), 1);
    assert_eq!(result.elapsed, DEADLINE);
    assert!(result.overall_deadline_reached);
    assert!(!result.collection_complete);
    assert!(matches!(
        result.canisters[0].outcome,
        PlacementOutcome::Read { .. }
    ));
    assert!(matches!(
        result.canisters[1].outcome,
        PlacementOutcome::OverallDeadline
    ));
    assert!(matches!(
        result.canisters[2].outcome,
        PlacementOutcome::NotStarted
    ));
}

#[tokio::test(start_paused = true)]
async fn caller_cancellation_drops_active_read_without_starting_tail() {
    let calls = Cell::new(0);
    let dropped = Cell::new(0);
    let ids = [id(1), id(2)];
    let work = collect(&ids, 10, |_| {
        calls.set(calls.get() + 1);
        let dropped = &dropped;

        async move {
            let _guard = Guard(dropped);

            pending().await
        }
    });
    assert!(
        tokio::time::timeout(Duration::from_secs(1), work)
            .await
            .is_err()
    );
    assert_eq!(calls.get(), 1);
    assert_eq!(dropped.get(), 1);
}

#[tokio::test]
async fn all_failed_is_partial_and_retains_typed_causes() {
    let result = collect(&[id(1), id(2)], 10, |_| async {
        Err(PlacementError::Read(ReadError::Deadline))
    })
    .await;
    assert!(!result.collection_complete);
    assert!(!result.overall_deadline_reached);
    assert_eq!(result.canisters.len(), 2);
    assert!(matches!(
        result.canisters[0].outcome,
        PlacementOutcome::Failed {
            error: PlacementError::Read(ReadError::Deadline)
        }
    ));
    let json = serde_json::to_value(result).unwrap();
    assert_eq!(
        json["canisters"][1]["outcome"]["error"]["kind"],
        "read_deadline"
    );
}
