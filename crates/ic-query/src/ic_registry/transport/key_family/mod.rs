use super::history_cache::{
    HISTORY_PUBLICATION_PAGE_INTERVAL, HistoryCacheObservation, MAX_HISTORY_BYTES,
    MAX_HISTORY_CHECKPOINTS, MAX_HISTORY_PAGE_BYTES, MAX_HISTORY_PAGES, RegistryHistoryPage,
};
use super::{
    RegistryQueryCounter, SubnetCatalogProgressPhase, decode_message, registry_error_code,
};
use crate::ic_registry::RegistryFetchError;
use ic_host_tools::registry::proto::{
    HighCapacityRegistryGetChangesSinceResponse, RegistryGetChangesSinceRequest,
    high_capacity_registry_value,
};
use ic_platform::agent::Agent;
use prost::Message;
use std::{
    collections::{BTreeMap, BTreeSet, btree_map::Entry},
    sync::Arc,
};

const MAX_REGISTRY_DELTA_KEYS: usize = 100_000;
const MAX_REGISTRY_DELTA_VALUES: usize = 1_000_000;
const MAX_REGISTRY_KEY_BYTES: usize = 1_024;

pub(in crate::ic_registry) async fn get_registry_key_family_counted(
    agent: &Agent,
    registry_canister: &ic_platform::candid::contract::Principal,
    prefix: &str,
    registry_version: u64,
    counter: &RegistryQueryCounter,
) -> Result<Vec<String>, RegistryFetchError> {
    collect_key_family(prefix, registry_version, counter, |cursor| {
        get_changes_since(agent, registry_canister, cursor, counter)
    })
    .await
}

pub(super) async fn collect_key_family<F, Fut>(
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
    let mut collected_pages: usize = 0;
    let mut family = RegistryKeyFamilyState::new(prefix, registry_version);
    let checkpoint_key = (counter.endpoint.clone(), prefix.to_string());
    let pages =
        if let Some(checkpoint) = restore_checkpoint(counter, &checkpoint_key, registry_version)? {
            cursor = checkpoint.version;
            family.states = checkpoint.states.clone();
            family.delta_key_count = checkpoint.delta_key_count;
            family.value_count = checkpoint.value_count;
            checkpoint.pages.clone()
        } else {
            counter
                .acquisition
                .as_ref()
                .and_then(|context| context.history_cache.as_ref())
                .map(|_| Vec::new())
        };
    let mut recording = RegistryHistoryRecording::new(pages);
    counter.emit(SubnetCatalogProgressPhase::History {
        registry_version,
        through_version: cursor,
        reused: cursor != 0,
    });
    while cursor < registry_version {
        let mut response = page(cursor).await?;
        let encoded = recording.prepare(&mut response);
        cursor = family.apply_page(response, cursor)?.min(registry_version);
        collected_pages += 1;
        recording.record(cursor, encoded);
        // Publish only after the entire page passes continuity and content validation.
        if let Some(context) = &counter.acquisition {
            let checkpoint = Arc::new(RegistryKeyFamilyCheckpoint {
                version: cursor,
                states: family.states.clone(),
                delta_key_count: family.delta_key_count,
                value_count: family.value_count,
                pages: recording.pages.clone(),
            });
            remember_checkpoint(counter, &checkpoint_key, Arc::clone(&checkpoint));
            if let Some(cache) = &context.history_cache
                && (collected_pages == 1
                    || collected_pages.is_multiple_of(HISTORY_PUBLICATION_PAGE_INTERVAL)
                    || cursor == registry_version)
            {
                let observation = cache.publish(&counter.endpoint, prefix, &checkpoint)?;
                emit_history_cache(counter, observation);
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

struct RegistryHistoryRecording {
    pages: Option<Vec<RegistryHistoryPage>>,
    retained_bytes: usize,
}

impl RegistryHistoryRecording {
    fn new(pages: Option<Vec<RegistryHistoryPage>>) -> Self {
        let retained_bytes = pages.as_ref().map_or(0, |pages| {
            pages.iter().map(|page| page.response_hex.len()).sum()
        });
        Self {
            pages,
            retained_bytes,
        }
    }

    fn prepare(
        &self,
        response: &mut HighCapacityRegistryGetChangesSinceResponse,
    ) -> Option<Vec<u8>> {
        self.pages.as_ref().map(|_| {
            normalize_history_response(response);
            response.encode_to_vec()
        })
    }

    // Commit normalized bytes only after the entire page has passed family admission.
    fn record(&mut self, through_version: u64, encoded: Option<Vec<u8>>) {
        let (Some(pages), Some(encoded)) = (self.pages.as_mut(), encoded) else {
            return;
        };
        let page_bytes = encoded.len().saturating_mul(2);
        if pages.len() >= MAX_HISTORY_PAGES
            || encoded.len() > MAX_HISTORY_PAGE_BYTES
            || self.retained_bytes.saturating_add(page_bytes)
                > usize::try_from(MAX_HISTORY_BYTES).expect("history byte ceiling fits usize")
        {
            self.pages = None;
            self.retained_bytes = 0;
            return;
        }
        pages.push(RegistryHistoryPage {
            through_version,
            response_hex: crate::hex::hex_bytes(&encoded).into(),
        });
        self.retained_bytes += page_bytes;
    }
}

fn restore_checkpoint(
    counter: &RegistryQueryCounter,
    key: &(String, String),
    pin: u64,
) -> Result<Option<Arc<RegistryKeyFamilyCheckpoint>>, RegistryFetchError> {
    let Some(context) = &counter.acquisition else {
        return Ok(None);
    };
    let retained = context
        .history
        .lock()
        .expect("history checkpoint lock")
        .get(key)
        .filter(|checkpoint| checkpoint.version <= pin)
        .cloned();
    if retained.is_some() {
        return Ok(retained);
    }
    let checkpoint = if let Some(cache) = &context.history_cache {
        let (checkpoint, observation) = cache.load(&key.0, &key.1, pin)?;
        emit_history_cache(counter, observation);
        checkpoint.map(Arc::new)
    } else {
        None
    };
    if let Some(checkpoint) = &checkpoint {
        remember_checkpoint(counter, key, Arc::clone(checkpoint));
    }
    Ok(checkpoint)
}

fn remember_checkpoint(
    counter: &RegistryQueryCounter,
    key: &(String, String),
    checkpoint: Arc<RegistryKeyFamilyCheckpoint>,
) {
    let Some(context) = &counter.acquisition else {
        return;
    };
    let mut history = context.history.lock().expect("history checkpoint lock");
    if (history.contains_key(key) || history.len() < MAX_HISTORY_CHECKPOINTS)
        && history
            .get(key)
            .is_none_or(|old| old.version <= checkpoint.version)
    {
        history.insert(key.clone(), checkpoint);
    }
}

async fn get_changes_since(
    agent: &Agent,
    registry_canister: &ic_platform::candid::contract::Principal,
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
    pub(super) version: u64,
    states: BTreeMap<String, (u64, bool)>,
    pub(super) pages: Option<Vec<RegistryHistoryPage>>,
}

impl RegistryKeyFamilyCheckpoint {
    pub(super) fn replay(
        prefix: &str,
        pages: Vec<RegistryHistoryPage>,
    ) -> Result<Self, RegistryFetchError> {
        if pages.is_empty() || pages.len() > MAX_HISTORY_PAGES {
            return Err(RegistryFetchError::InvalidRegistryKeyFamily {
                reason: "invalid history transcript page count".to_string(),
            });
        }
        let mut family = RegistryKeyFamilyState::new(prefix, 0);
        let mut cursor = 0;
        for page in &pages {
            if page.through_version <= cursor
                || page.response_hex.len() > MAX_HISTORY_PAGE_BYTES * 2
            {
                return Err(RegistryFetchError::InvalidRegistryKeyFamily {
                    reason: "invalid history page watermark or byte length".to_string(),
                });
            }
            let bytes = crate::hex::decode_lowercase_hex(&page.response_hex).ok_or_else(|| {
                RegistryFetchError::InvalidRegistryKeyFamily {
                    reason: "history page is not canonical lowercase hexadecimal".to_string(),
                }
            })?;
            let mut response = decode_message::<HighCapacityRegistryGetChangesSinceResponse>(
                "Registry history page",
                &bytes,
            )?;
            normalize_history_response(&mut response);
            if response.encode_to_vec() != bytes {
                return Err(RegistryFetchError::InvalidRegistryKeyFamily {
                    reason: "history page is not canonical normalized mutation evidence"
                        .to_string(),
                });
            }
            family.registry_version = page.through_version;
            let end = family.apply_page(response, cursor)?;
            if end < page.through_version {
                return Err(RegistryFetchError::IncompleteRegistryChanges {
                    requested_version: page.through_version,
                    observed_version: end,
                });
            }
            cursor = page.through_version;
        }
        Ok(Self {
            version: cursor,
            states: family.states,
            delta_key_count: family.delta_key_count,
            value_count: family.value_count,
            pages: Some(pages),
        })
    }
}

fn normalize_history_response(response: &mut HighCapacityRegistryGetChangesSinceResponse) {
    for delta in &mut response.deltas {
        for value in &mut delta.values {
            value.timestamp_nanoseconds = 0;
            match &mut value.content {
                Some(high_capacity_registry_value::Content::Value(bytes)) => bytes.clear(),
                Some(high_capacity_registry_value::Content::LargeValueChunkKeys(_)) => {
                    value.content = Some(high_capacity_registry_value::Content::Value(Vec::new()));
                }
                _ => {}
            }
        }
    }
}

fn emit_history_cache(counter: &RegistryQueryCounter, observation: HistoryCacheObservation) {
    if let Some(cache) = counter
        .acquisition
        .as_ref()
        .and_then(|context| context.history_cache.as_ref())
    {
        counter.emit(SubnetCatalogProgressPhase::HistoryCache {
            path: cache.paths().0.to_path_buf(),
            disposition: observation.disposition,
            through_version: observation.through_version,
            reason: observation.reason,
        });
    }
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
        charge_key_family_count(
            &mut self.delta_key_count,
            response.deltas.len(),
            "delta_keys",
            MAX_REGISTRY_DELTA_KEYS,
        )?;

        let mut page_versions = BTreeSet::new();
        let mut page_end = cursor;
        for delta in response.deltas {
            charge_key_family_count(
                &mut self.value_count,
                delta.values.len(),
                "delta_values",
                MAX_REGISTRY_DELTA_VALUES,
            )?;
            enforce_key_family_limit("key_bytes", MAX_REGISTRY_KEY_BYTES, delta.key.len())?;
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
                if value.version > response.version {
                    return Err(RegistryFetchError::InvalidRegistryKeyFamily {
                        reason: "mutation version exceeds the response's latest Registry version"
                            .to_string(),
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
                self.apply_mutation(key, value.version, value.content)?;
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

    #[expect(
        clippy::needless_pass_by_value,
        reason = "The observed protobuf content is a single-use wire value owned by this Registry observation."
    )]
    fn apply_mutation(
        &mut self,
        key: &str,
        version: u64,
        content: Option<high_capacity_registry_value::Content>,
    ) -> Result<(), RegistryFetchError> {
        let present = match content {
            Some(
                high_capacity_registry_value::Content::Value(_)
                | high_capacity_registry_value::Content::LargeValueChunkKeys(_),
            ) => true,
            Some(high_capacity_registry_value::Content::DeletionMarker(true)) => false,
            Some(high_capacity_registry_value::Content::DeletionMarker(false)) => {
                return Err(RegistryFetchError::InvalidRegistryKeyFamily {
                    reason: format!("key {key:?} version {version} has a false deletion marker"),
                });
            }
            None => {
                return Err(RegistryFetchError::InvalidRegistryKeyFamily {
                    reason: format!(
                        "key {key:?} version {version} has no value or deletion marker"
                    ),
                });
            }
        };
        match self.states.entry(key.to_string()) {
            Entry::Vacant(entry) => {
                entry.insert((version, present));
            }
            Entry::Occupied(mut entry) => {
                let (current_version, _) = *entry.get();
                if version == current_version {
                    return Err(RegistryFetchError::InvalidRegistryKeyFamily {
                        reason: format!("key {key:?} has duplicate mutations at version {version}"),
                    });
                }
                if version > current_version {
                    entry.insert((version, present));
                }
            }
        }
        Ok(())
    }

    fn into_keys(self) -> Vec<String> {
        self.states
            .into_iter()
            .filter_map(|(key, (_, present))| present.then_some(key))
            .collect()
    }
}

fn charge_key_family_count(
    count: &mut usize,
    additional: usize,
    field: &'static str,
    maximum: usize,
) -> Result<(), RegistryFetchError> {
    *count = count
        .checked_add(additional)
        .ok_or(RegistryFetchError::RegistryKeyFamilyLimit {
            field,
            maximum,
            actual: usize::MAX,
        })?;
    enforce_key_family_limit(field, maximum, *count)
}

const fn enforce_key_family_limit(
    field: &'static str,
    maximum: usize,
    actual: usize,
) -> Result<(), RegistryFetchError> {
    if actual > maximum {
        return Err(RegistryFetchError::RegistryKeyFamilyLimit {
            field,
            maximum,
            actual,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
