use super::*;
use ic_host_tools::registry::proto::{HighCapacityRegistryDelta, HighCapacityRegistryValue};
use std::{
    future::Future,
    sync::{Arc, Mutex},
    task::Context,
};

fn value(version: u64) -> HighCapacityRegistryValue {
    HighCapacityRegistryValue {
        version,
        content: Some(high_capacity_registry_value::Content::Value(Vec::new())),
        timestamp_nanoseconds: version,
    }
}

fn deletion(version: u64) -> HighCapacityRegistryValue {
    HighCapacityRegistryValue {
        version,
        content: Some(high_capacity_registry_value::Content::DeletionMarker(true)),
        timestamp_nanoseconds: version,
    }
}

fn response(
    version: u64,
    deltas: Vec<HighCapacityRegistryDelta>,
) -> HighCapacityRegistryGetChangesSinceResponse {
    HighCapacityRegistryGetChangesSinceResponse {
        error: None,
        version,
        deltas,
    }
}

#[test]
fn history_recording_stops_permanently_at_the_page_ceiling() {
    let mut recording = RegistryHistoryRecording::new(Some(Vec::new()));
    let mut page = response(
        1,
        vec![HighCapacityRegistryDelta {
            key: b"canister_ranges_00".to_vec(),
            values: vec![value(1)],
        }],
    );
    let encoded = recording.prepare(&mut page);
    recording.record(1, encoded);
    let first = recording.pages.expect("recorded validated page").remove(0);
    let mut resumed = RegistryHistoryRecording::new(Some(vec![first; MAX_HISTORY_PAGES]));
    let encoded = resumed.prepare(&mut page);
    resumed.record(2, encoded);
    assert!(resumed.pages.is_none());
    assert!(resumed.prepare(&mut page).is_none());
}

#[test]
fn checkpoints_resume_only_the_same_endpoint_and_forward_pin() {
    futures::executor::block_on(async {
        let context = std::sync::Arc::default();
        let first = RegistryQueryCounter::with_acquisition(
            "https://a.example".into(),
            std::sync::Arc::clone(&context),
        );
        let second = RegistryQueryCounter::with_acquisition("https://b.example".into(), context);
        let page = |version, values| {
            response(
                version,
                vec![HighCapacityRegistryDelta {
                    key: b"canister_ranges_00".to_vec(),
                    values,
                }],
            )
        };
        let keys = collect_key_family("canister_ranges_", 2, &first, |cursor| {
            assert_eq!(cursor, 0);
            std::future::ready(Ok(page(2, vec![value(1), value(2)])))
        })
        .await
        .unwrap();
        assert_eq!(keys, ["canister_ranges_00"]);
        let keys = collect_key_family("canister_ranges_", 2, &first, |_| {
            std::future::ready(Err(RegistryFetchError::IncompleteRegistryChanges {
                requested_version: 2,
                observed_version: 0,
            }))
        })
        .await
        .unwrap();
        assert_eq!(keys, ["canister_ranges_00"]);
        let keys = collect_key_family("canister_ranges_", 3, &first, |cursor| {
            assert_eq!(cursor, 2);
            std::future::ready(Ok(page(3, vec![deletion(3)])))
        })
        .await
        .unwrap();
        assert_eq!(keys, Vec::<String>::new());
        for counter in [&first, &second] {
            let keys = collect_key_family("canister_ranges_", 1, counter, |cursor| {
                assert_eq!(cursor, 0);
                std::future::ready(Ok(page(1, vec![value(1)])))
            })
            .await
            .unwrap();
            assert_eq!(keys, ["canister_ranges_00"]);
        }
    });
}

#[test]
fn failed_page_preserves_last_complete_prefix_for_retry() {
    futures::executor::block_on(async {
        let counter = RegistryQueryCounter::with_acquisition(
            "https://a.example".into(),
            std::sync::Arc::default(),
        );
        let failed = collect_key_family("canister_ranges_", 3, &counter, |cursor| {
            std::future::ready(Ok(response(
                3,
                vec![HighCapacityRegistryDelta {
                    key: b"canister_ranges_00".to_vec(),
                    values: if cursor == 0 {
                        vec![value(1)]
                    } else {
                        vec![deletion(3)]
                    },
                }],
            )))
        })
        .await;
        assert!(matches!(
            failed,
            Err(RegistryFetchError::InvalidRegistryKeyFamily { .. })
        ));
        let keys = collect_key_family("canister_ranges_", 3, &counter, |cursor| {
            assert_eq!(cursor, 1);
            std::future::ready(Ok(response(
                3,
                vec![HighCapacityRegistryDelta {
                    key: b"canister_ranges_00".to_vec(),
                    values: vec![value(2), deletion(3)],
                }],
            )))
        })
        .await
        .unwrap();
        assert_eq!(keys, Vec::<String>::new());
    });
}

#[tokio::test(flavor = "current_thread")]
async fn cancellation_keeps_validated_history_and_reports_endpoint_progress() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let observed = Arc::clone(&events);
    let acquisition = super::super::RegistryAcquisition {
        progress: Some(Arc::new(move |event| observed.lock().unwrap().push(event))),
        ..Default::default()
    };
    let counter =
        RegistryQueryCounter::with_acquisition("https://a.example".into(), Arc::new(acquisition));
    let mut future = Box::pin(collect_key_family(
        "canister_ranges_",
        3,
        &counter,
        |cursor| async move {
            if cursor != 0 {
                return std::future::pending().await;
            }
            Ok(response(
                3,
                vec![HighCapacityRegistryDelta {
                    key: b"canister_ranges_00".to_vec(),
                    values: vec![value(1)],
                }],
            ))
        },
    ));
    let waker = futures::task::noop_waker();
    assert!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(&waker))
            .is_pending()
    );
    drop(future);
    let keys = futures::executor::block_on(collect_key_family(
        "canister_ranges_",
        3,
        &counter,
        |cursor| {
            assert_eq!(cursor, 1);
            std::future::ready(Ok(response(
                3,
                vec![HighCapacityRegistryDelta {
                    key: b"canister_ranges_00".to_vec(),
                    values: vec![value(2), value(3)],
                }],
            )))
        },
    ))
    .unwrap();
    assert_eq!(keys, ["canister_ranges_00"]);
    let events = events.lock().unwrap();
    assert!(
        events
            .iter()
            .all(|event| event.endpoint == "https://a.example")
    );
    assert_eq!(
        events.iter().map(|event| &event.phase).collect::<Vec<_>>(),
        vec![
            &SubnetCatalogProgressPhase::History {
                registry_version: 3,
                through_version: 0,
                reused: false
            },
            &SubnetCatalogProgressPhase::History {
                registry_version: 3,
                through_version: 1,
                reused: false
            },
            &SubnetCatalogProgressPhase::History {
                registry_version: 3,
                through_version: 1,
                reused: true
            },
            &SubnetCatalogProgressPhase::History {
                registry_version: 3,
                through_version: 3,
                reused: false
            },
        ]
    );
}

#[test]
fn checkpoint_watermark_stops_at_pin_even_when_page_contains_later_mutations() {
    futures::executor::block_on(async {
        let counter =
            RegistryQueryCounter::with_acquisition("https://a.example".into(), Arc::default());
        let keys = collect_key_family("canister_ranges_", 1, &counter, |_| {
            std::future::ready(Ok(response(
                3,
                vec![HighCapacityRegistryDelta {
                    key: b"canister_ranges_00".to_vec(),
                    values: vec![value(1), deletion(2), value(3)],
                }],
            )))
        })
        .await
        .unwrap();
        assert_eq!(keys, ["canister_ranges_00"]);
        let keys = collect_key_family("canister_ranges_", 2, &counter, |cursor| {
            assert_eq!(cursor, 1);
            std::future::ready(Ok(response(
                3,
                vec![HighCapacityRegistryDelta {
                    key: b"canister_ranges_00".to_vec(),
                    values: vec![deletion(2), value(3)],
                }],
            )))
        })
        .await
        .unwrap();
        assert_eq!(keys, Vec::<String>::new());
    });
}

#[test]
fn reconstructs_present_family_at_pinned_version() {
    let keys = key_family_at_version(
        response(
            12,
            vec![
                HighCapacityRegistryDelta {
                    key: b"canister_ranges_00".to_vec(),
                    values: vec![value(4), deletion(9)],
                },
                HighCapacityRegistryDelta {
                    key: b"canister_ranges_01".to_vec(),
                    values: vec![value(7), deletion(11)],
                },
                HighCapacityRegistryDelta {
                    key: b"unrelated".to_vec(),
                    values: (1..=11).map(value).collect(),
                },
            ],
        ),
        "canister_ranges_",
        10,
    )
    .expect("family");

    assert_eq!(keys, vec!["canister_ranges_01"]);
}

#[test]
fn applies_contiguous_pages_until_the_pinned_version() {
    let mut family = RegistryKeyFamilyState::new("canister_ranges_", 4);
    let first_end = family
        .apply_page(
            response(
                4,
                vec![
                    HighCapacityRegistryDelta {
                        key: b"canister_ranges_00".to_vec(),
                        values: vec![value(2)],
                    },
                    HighCapacityRegistryDelta {
                        key: b"unrelated".to_vec(),
                        values: vec![value(1), value(2)],
                    },
                ],
            ),
            0,
        )
        .expect("first page");
    assert_eq!(first_end, 2);

    let second_end = family
        .apply_page(
            response(
                4,
                vec![
                    HighCapacityRegistryDelta {
                        key: b"canister_ranges_00".to_vec(),
                        values: vec![deletion(4)],
                    },
                    HighCapacityRegistryDelta {
                        key: b"unrelated".to_vec(),
                        values: vec![value(3), value(4)],
                    },
                ],
            ),
            first_end,
        )
        .expect("second page");

    assert_eq!(second_end, 4);
    assert_eq!(family.into_keys(), Vec::<String>::new());
}

#[test]
fn rejects_a_response_that_does_not_reach_the_pinned_version() {
    assert!(matches!(
        key_family_at_version(response(9, Vec::new()), "canister_ranges_", 10),
        Err(RegistryFetchError::IncompleteRegistryChanges {
            requested_version: 10,
            observed_version: 9,
        })
    ));
}

#[test]
fn rejects_a_noncontiguous_or_no_progress_page() {
    let mut family = RegistryKeyFamilyState::new("canister_ranges_", 4);
    assert!(matches!(
        family.apply_page(
            response(
                4,
                vec![HighCapacityRegistryDelta {
                    key: b"unrelated".to_vec(),
                    values: vec![value(1), value(3)],
                }]
            ),
            0,
        ),
        Err(RegistryFetchError::InvalidRegistryKeyFamily { .. })
    ));

    let mut family = RegistryKeyFamilyState::new("canister_ranges_", 4);
    assert!(matches!(
        family.apply_page(response(4, Vec::new()), 0),
        Err(RegistryFetchError::IncompleteRegistryChanges {
            requested_version: 4,
            observed_version: 0,
        })
    ));
}

#[test]
fn rejects_exceeded_page_budgets_before_applying_mutations() {
    for (keys, values, key, field, maximum) in [
        (
            MAX_REGISTRY_DELTA_KEYS,
            0,
            b"canister_ranges_00".to_vec(),
            "delta_keys",
            MAX_REGISTRY_DELTA_KEYS,
        ),
        (
            0,
            MAX_REGISTRY_DELTA_VALUES,
            b"canister_ranges_00".to_vec(),
            "delta_values",
            MAX_REGISTRY_DELTA_VALUES,
        ),
        (
            0,
            0,
            vec![b'x'; MAX_REGISTRY_KEY_BYTES + 1],
            "key_bytes",
            MAX_REGISTRY_KEY_BYTES,
        ),
    ] {
        let mut family = RegistryKeyFamilyState::new("canister_ranges_", 1);
        family.delta_key_count = keys;
        family.value_count = values;
        let error = family
            .apply_page(
                response(
                    1,
                    vec![HighCapacityRegistryDelta {
                        key,
                        values: vec![value(1)],
                    }],
                ),
                0,
            )
            .unwrap_err();
        assert!(matches!(error, RegistryFetchError::RegistryKeyFamilyLimit {
                field: actual_field, maximum: actual_maximum, actual,
            } if actual_field == field && actual_maximum == maximum && actual == maximum + 1));
        assert!(family.states.is_empty());
    }
}

#[test]
fn rejects_contradictory_same_version_mutations() {
    let error = key_family_at_version(
        response(
            10,
            vec![HighCapacityRegistryDelta {
                key: b"canister_ranges_00".to_vec(),
                values: vec![value(8), deletion(8)],
            }],
        ),
        "canister_ranges_",
        10,
    )
    .expect_err("contradictory evidence");

    assert!(matches!(
        error,
        RegistryFetchError::InvalidRegistryKeyFamily { .. }
    ));
}

fn key_family_at_version(
    response: HighCapacityRegistryGetChangesSinceResponse,
    prefix: &str,
    registry_version: u64,
) -> Result<Vec<String>, RegistryFetchError> {
    let mut family = RegistryKeyFamilyState::new(prefix, registry_version);
    let page_end = family.apply_page(response, 0)?;
    if page_end < registry_version {
        return Err(RegistryFetchError::IncompleteRegistryChanges {
            requested_version: registry_version,
            observed_version: page_end,
        });
    }
    Ok(family.into_keys())
}
