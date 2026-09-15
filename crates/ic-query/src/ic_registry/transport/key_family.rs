use super::{RegistryQueryCounter, SubnetCatalogProgressPhase, decode_message};
use crate::ic_registry::{
    RegistryFetchError,
    proto::{
        HighCapacityRegistryGetChangesSinceResponse, RegistryErrorCode,
        RegistryGetChangesSinceRequest, high_capacity_registry_value,
    },
};
use ic_agent::Agent;
use prost::Message;
use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

const MAX_HISTORY_CHECKPOINTS: usize = 8;

const MAX_REGISTRY_DELTA_KEYS: usize = 100_000;
const MAX_REGISTRY_DELTA_VALUES: usize = 1_000_000;
const MAX_REGISTRY_KEY_BYTES: usize = 1_024;

pub(in crate::ic_registry) async fn get_registry_key_family_counted(
    agent: &Agent,
    registry_canister: &candid::Principal,
    prefix: &str,
    registry_version: u64,
    counter: &RegistryQueryCounter,
) -> Result<Vec<String>, RegistryFetchError> {
    collect_key_family(prefix, registry_version, counter, |cursor| {
        get_changes_since(agent, registry_canister, cursor, counter)
    })
    .await
}

async fn collect_key_family<F, Fut>(
    prefix: &str,
    registry_version: u64,
    counter: &RegistryQueryCounter,
    mut page: F,
) -> Result<Vec<String>, RegistryFetchError>
where
    F: FnMut(u64) -> Fut,
    Fut: std::future::Future<
            Output = Result<HighCapacityRegistryGetChangesSinceResponse, RegistryFetchError>,
        >,
{
    let mut cursor = 0;
    let mut family = RegistryKeyFamilyState::new(prefix, registry_version);
    let checkpoint_key = (counter.endpoint.clone(), prefix.to_string());
    if let Some(context) = &counter.acquisition {
        let history = context.history.lock().expect("history checkpoint lock");
        if let Some(checkpoint) = history.get(&checkpoint_key)
            && checkpoint.version <= registry_version
        {
            cursor = checkpoint.version;
            family.states.clone_from(&checkpoint.states);
            family.delta_key_count = checkpoint.delta_key_count;
            family.value_count = checkpoint.value_count;
        }
    }
    counter.emit(SubnetCatalogProgressPhase::History {
        registry_version,
        through_version: cursor,
        reused: cursor != 0,
    });
    while cursor < registry_version {
        let response = page(cursor).await?;
        cursor = family.apply_page(response, cursor)?.min(registry_version);
        // Publish only after the entire page passes continuity and content validation.
        if let Some(context) = &counter.acquisition {
            let mut history = context.history.lock().expect("history checkpoint lock");
            if (history.contains_key(&checkpoint_key) || history.len() < MAX_HISTORY_CHECKPOINTS)
                && history
                    .get(&checkpoint_key)
                    .is_none_or(|old| old.version <= cursor)
            {
                history.insert(
                    checkpoint_key.clone(),
                    RegistryKeyFamilyCheckpoint {
                        version: cursor,
                        states: family.states.clone(),
                        delta_key_count: family.delta_key_count,
                        value_count: family.value_count,
                    },
                );
            }
        }
        counter.emit(SubnetCatalogProgressPhase::History {
            registry_version,
            through_version: cursor,
            reused: false,
        });
    }
    Ok(family.into_keys())
}

async fn get_changes_since(
    agent: &Agent,
    registry_canister: &candid::Principal,
    version: u64,
    counter: &RegistryQueryCounter,
) -> Result<HighCapacityRegistryGetChangesSinceResponse, RegistryFetchError> {
    let mut arg = Vec::new();
    RegistryGetChangesSinceRequest { version }
        .encode(&mut arg)
        .map_err(|error| RegistryFetchError::ProtobufEncode {
            message: "RegistryGetChangesSinceRequest",
            reason: error.to_string(),
        })?;
    let bytes = super::query::query(
        agent,
        registry_canister,
        "get_changes_since",
        arg,
        Some(counter),
    )
    .await
    .map_err(|error| RegistryFetchError::AgentCall {
        method: "get_changes_since",
        reason: error.to_string(),
    })?;
    decode_message::<HighCapacityRegistryGetChangesSinceResponse>(
        "HighCapacityRegistryGetChangesSinceResponse",
        &bytes,
    )
}

///
/// RegistryKeyFamilyCheckpoint
///
/// Complete endpoint-local key-family state through a validated history prefix.
///

pub(super) struct RegistryKeyFamilyCheckpoint {
    delta_key_count: usize,
    value_count: usize,
    version: u64,
    states: BTreeMap<String, (u64, bool)>,
}

struct RegistryKeyFamilyState<'a> {
    prefix: &'a [u8],
    registry_version: u64,
    states: BTreeMap<String, (u64, bool)>,
    delta_key_count: usize,
    value_count: usize,
}

impl<'a> RegistryKeyFamilyState<'a> {
    const fn new(prefix: &'a str, registry_version: u64) -> Self {
        Self {
            prefix: prefix.as_bytes(),
            registry_version,
            states: BTreeMap::new(),
            delta_key_count: 0,
            value_count: 0,
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "each paginated delta page is validated and applied as one fail-closed unit"
    )]
    fn apply_page(
        &mut self,
        response: HighCapacityRegistryGetChangesSinceResponse,
        cursor: u64,
    ) -> Result<u64, RegistryFetchError> {
        if let Some(error) = response.error {
            return Err(RegistryFetchError::RegistryChanges {
                code: registry_error_code(error.code).to_string(),
                reason: error.reason,
            });
        }
        if response.version < self.registry_version {
            return Err(RegistryFetchError::IncompleteRegistryChanges {
                requested_version: self.registry_version,
                observed_version: response.version,
            });
        }
        self.delta_key_count = self
            .delta_key_count
            .checked_add(response.deltas.len())
            .ok_or(RegistryFetchError::RegistryKeyFamilyLimit {
                field: "delta_keys",
                maximum: MAX_REGISTRY_DELTA_KEYS,
                actual: usize::MAX,
            })?;
        if self.delta_key_count > MAX_REGISTRY_DELTA_KEYS {
            return Err(RegistryFetchError::RegistryKeyFamilyLimit {
                field: "delta_keys",
                maximum: MAX_REGISTRY_DELTA_KEYS,
                actual: self.delta_key_count,
            });
        }

        let mut page_versions = BTreeSet::new();
        let mut page_end = cursor;
        for delta in response.deltas {
            self.value_count = self.value_count.checked_add(delta.values.len()).ok_or(
                RegistryFetchError::RegistryKeyFamilyLimit {
                    field: "delta_values",
                    maximum: MAX_REGISTRY_DELTA_VALUES,
                    actual: usize::MAX,
                },
            )?;
            if self.value_count > MAX_REGISTRY_DELTA_VALUES {
                return Err(RegistryFetchError::RegistryKeyFamilyLimit {
                    field: "delta_values",
                    maximum: MAX_REGISTRY_DELTA_VALUES,
                    actual: self.value_count,
                });
            }
            if delta.key.len() > MAX_REGISTRY_KEY_BYTES {
                return Err(RegistryFetchError::RegistryKeyFamilyLimit {
                    field: "key_bytes",
                    maximum: MAX_REGISTRY_KEY_BYTES,
                    actual: delta.key.len(),
                });
            }
            let matching_key = if delta.key.starts_with(self.prefix) {
                Some(String::from_utf8(delta.key).map_err(|error| {
                    RegistryFetchError::InvalidRegistryKeyFamily {
                        reason: format!("matching key is not UTF-8: {error}"),
                    }
                })?)
            } else {
                None
            };
            for value in delta.values {
                if value.version <= cursor {
                    return Err(RegistryFetchError::InvalidRegistryKeyFamily {
                        reason: format!(
                            "get_changes_since page after version {cursor} returned mutation version {}",
                            value.version
                        ),
                    });
                }
                page_versions.insert(value.version);
                page_end = page_end.max(value.version);
                if value.version > self.registry_version {
                    continue;
                }
                let Some(key) = matching_key.as_ref() else {
                    continue;
                };
                let present = match value.content {
                    Some(
                        high_capacity_registry_value::Content::Value(_)
                        | high_capacity_registry_value::Content::LargeValueChunkKeys(_),
                    ) => true,
                    Some(high_capacity_registry_value::Content::DeletionMarker(true)) => false,
                    Some(high_capacity_registry_value::Content::DeletionMarker(false)) => {
                        return Err(RegistryFetchError::InvalidRegistryKeyFamily {
                            reason: format!(
                                "key {key:?} version {} has a false deletion marker",
                                value.version
                            ),
                        });
                    }
                    None => {
                        return Err(RegistryFetchError::InvalidRegistryKeyFamily {
                            reason: format!(
                                "key {key:?} version {} has no value or deletion marker",
                                value.version
                            ),
                        });
                    }
                };
                match self.states.entry(key.clone()) {
                    Entry::Vacant(entry) => {
                        entry.insert((value.version, present));
                    }
                    Entry::Occupied(mut entry) => {
                        let (current_version, _) = *entry.get();
                        if value.version == current_version {
                            return Err(RegistryFetchError::InvalidRegistryKeyFamily {
                                reason: format!(
                                    "key {key:?} has duplicate mutations at version {}",
                                    value.version
                                ),
                            });
                        }
                        if value.version > current_version {
                            entry.insert((value.version, present));
                        }
                    }
                }
            }
        }

        if page_end == cursor {
            return Err(RegistryFetchError::IncompleteRegistryChanges {
                requested_version: self.registry_version,
                observed_version: cursor,
            });
        }
        let expected_version_count = usize::try_from(page_end - cursor).map_err(|_| {
            RegistryFetchError::InvalidRegistryKeyFamily {
                reason: format!(
                    "Registry delta page version span {cursor}..={page_end} does not fit usize"
                ),
            }
        })?;
        if page_versions.len() != expected_version_count {
            return Err(RegistryFetchError::InvalidRegistryKeyFamily {
                reason: format!(
                    "Registry delta page after version {cursor} is not contiguous through version {page_end}"
                ),
            });
        }
        Ok(page_end)
    }

    fn into_keys(self) -> Vec<String> {
        self.states
            .into_iter()
            .filter_map(|(key, (_, present))| present.then_some(key))
            .collect()
    }
}

#[cfg(test)]
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

fn registry_error_code(code: i32) -> &'static str {
    match RegistryErrorCode::try_from(code).ok() {
        Some(RegistryErrorCode::MalformedMessage) => "malformed_message",
        Some(RegistryErrorCode::KeyNotPresent) => "key_not_present",
        Some(RegistryErrorCode::KeyAlreadyPresent) => "key_already_present",
        Some(RegistryErrorCode::VersionNotLatest) => "version_not_latest",
        Some(RegistryErrorCode::VersionBeyondLatest) => "version_beyond_latest",
        Some(RegistryErrorCode::Authorization) => "authorization",
        Some(RegistryErrorCode::InternalError) => "internal_error",
        None => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ic_registry::proto::{HighCapacityRegistryDelta, HighCapacityRegistryValue};
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
    fn checkpoints_resume_only_the_same_endpoint_and_forward_pin() {
        futures::executor::block_on(async {
            let context = std::sync::Arc::default();
            let first = RegistryQueryCounter::with_acquisition(
                "https://a.example".into(),
                std::sync::Arc::clone(&context),
            );
            let second =
                RegistryQueryCounter::with_acquisition("https://b.example".into(), context);
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
            assert!(keys.is_empty());
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
            assert!(keys.is_empty());
        });
    }

    #[test]
    fn cancellation_keeps_validated_history_and_reports_endpoint_progress() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let observed = Arc::clone(&events);
        let acquisition = super::super::RegistryAcquisition {
            progress: Some(Box::new(move |event| observed.lock().unwrap().push(event))),
            ..Default::default()
        };
        let counter = RegistryQueryCounter::with_acquisition(
            "https://a.example".into(),
            Arc::new(acquisition),
        );
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
            assert!(keys.is_empty());
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
        assert!(family.into_keys().is_empty());
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
}
