use super::{RegistryQueryCounter, decode_message};
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
    let mut cursor = 0;
    let mut family = RegistryKeyFamilyState::new(prefix, registry_version);
    while cursor < registry_version {
        let response = get_changes_since(agent, registry_canister, cursor, counter).await?;
        cursor = family.apply_page(response, cursor)?;
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
    counter.record_call();
    let bytes = agent
        .query(registry_canister, "get_changes_since")
        .with_arg(arg)
        .call()
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
