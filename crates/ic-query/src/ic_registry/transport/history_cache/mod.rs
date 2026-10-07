//! Module: ic_registry::transport::history_cache
//!
//! Responsibility: bounded, confined cross-process Registry history transcripts.
//! Does not own: catalog authority, live queries, or key-family replay validation.
//! Boundary: local-owner query evidence is checksummed, locked, and atomically published.

use super::key_family::RegistryKeyFamilyCheckpoint;
use crate::{
    cache_file::{
        BoundedManagedFileReadError, CacheFileError, HostCacheError, RefreshLockRequest,
        canonical_json_serialized_len, canonical_json_sha256, create_managed_parent_directory,
        read_bounded_managed_file, with_refresh_lock, write_managed_file_atomically,
    },
    hex::hex_bytes,
    subnet_catalog::{
        MAINNET_NETWORK, MAINNET_REGISTRY_CANISTER_ID, format_utc_timestamp_secs,
        parse_utc_timestamp_secs, subnet_catalog_history_lock_path, subnet_catalog_history_path,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

pub(super) const MAX_HISTORY_CHECKPOINTS: usize = 8;
pub(super) const MAX_HISTORY_PAGES: usize = 4_096;
pub(super) const MAX_HISTORY_BYTES: u64 = 64 * 1_024 * 1_024;
pub(super) const MAX_HISTORY_PAGE_BYTES: usize = crate::agent::MAX_IC_AGENT_RESPONSE_BODY_BYTES;
pub(super) const HISTORY_PUBLICATION_PAGE_INTERVAL: usize = 8;
const REGISTRY_HISTORY_SCHEMA_VERSION: u32 = 1;
const COMPONENT: &str = "Registry history";

///
/// RegistryHistoryCacheDisposition
///
/// Observable disk checkpoint handling, separate from catalog authority and freshness.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryHistoryCacheDisposition {
    /// No checkpoint exists for this exact endpoint and key family.
    Missing,
    /// A transcript was replayed and its completely validated prefix restored.
    Reused,
    /// Invalid local content was rejected before an authorized cold collection.
    Rejected,
    /// A completely validated prefix was atomically published.
    Published,
    /// Reuse or publication was skipped because of pin, retention, or writer policy.
    Skipped,
}

///
/// RegistryHistoryPage
///
/// Normalized protobuf mutation evidence through one completely validated watermark.
/// Values retain presence/deletion and versions, never catalog record payloads.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RegistryHistoryPage {
    pub through_version: u64,
    #[serde(
        serialize_with = "serialize_history_hex",
        deserialize_with = "deserialize_history_hex"
    )]
    pub response_hex: Arc<str>,
}

fn serialize_history_hex<S: serde::Serializer>(
    value: &str,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(value)
}

fn deserialize_history_hex<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Arc<str>, D::Error> {
    String::deserialize(deserializer).map(Into::into)
}

///
/// HistoryCacheObservation
///
/// One IO outcome delivered to the source's typed progress callback after locks release.
///

pub(super) struct HistoryCacheObservation {
    pub disposition: RegistryHistoryCacheDisposition,
    pub through_version: u64,
    pub reason: Option<String>,
}

impl HistoryCacheObservation {
    fn skipped(version: u64, reason: &str) -> Self {
        Self {
            disposition: RegistryHistoryCacheDisposition::Skipped,
            through_version: version,
            reason: Some(reason.to_string()),
        }
    }
}

///
/// RegistryHistoryCache
///
/// Caller-owned capability root for at most eight endpoint-isolated history transcripts.
///

#[derive(Clone)]
pub(super) struct RegistryHistoryCache {
    root: PathBuf,
    path: PathBuf,
    lock_path: PathBuf,
}

impl RegistryHistoryCache {
    pub(super) fn new(root: PathBuf) -> Self {
        Self {
            path: subnet_catalog_history_path(&root, MAINNET_NETWORK),
            lock_path: subnet_catalog_history_lock_path(&root, MAINNET_NETWORK),
            root,
        }
    }

    pub(super) fn paths(&self) -> (&Path, &Path) {
        (&self.path, &self.lock_path)
    }

    pub(super) fn load(
        &self,
        endpoint: &str,
        prefix: &str,
        pin: u64,
    ) -> Result<(Option<RegistryKeyFamilyCheckpoint>, HistoryCacheObservation), HostCacheError>
    {
        let (document, rejection) = self.read()?;
        if let Some(reason) = rejection {
            return Ok((
                None,
                HistoryCacheObservation {
                    disposition: RegistryHistoryCacheDisposition::Rejected,
                    through_version: 0,
                    reason: Some(reason),
                },
            ));
        }
        let Some(entry) = document
            .checkpoints
            .into_iter()
            .find(|entry| entry.endpoint == endpoint && entry.prefix == prefix)
        else {
            return Ok((
                None,
                HistoryCacheObservation {
                    disposition: RegistryHistoryCacheDisposition::Missing,
                    through_version: 0,
                    reason: None,
                },
            ));
        };
        if entry.version() > pin {
            return Ok((
                None,
                HistoryCacheObservation::skipped(
                    entry.version(),
                    "requested pin precedes the retained prefix; replaying from zero",
                ),
            ));
        }
        match RegistryKeyFamilyCheckpoint::replay(prefix, entry.pages) {
            Ok(checkpoint) => {
                let version = checkpoint.version;
                Ok((
                    Some(checkpoint),
                    HistoryCacheObservation {
                        disposition: RegistryHistoryCacheDisposition::Reused,
                        through_version: version,
                        reason: None,
                    },
                ))
            }
            Err(error) => Ok((
                None,
                HistoryCacheObservation {
                    disposition: RegistryHistoryCacheDisposition::Rejected,
                    through_version: 0,
                    reason: Some(error.to_string()),
                },
            )),
        }
    }

    pub(super) fn publish(
        &self,
        endpoint: &str,
        prefix: &str,
        checkpoint: &RegistryKeyFamilyCheckpoint,
    ) -> Result<HistoryCacheObservation, HostCacheError> {
        let Some(pages) = &checkpoint.pages else {
            return Ok(HistoryCacheObservation::skipped(
                checkpoint.version,
                "transcript retention limit reached",
            ));
        };
        create_managed_parent_directory(&self.root, &self.path).map_err(operation)?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| {
                HostCacheError::read_cache(
                    COMPONENT,
                    self.lock_path.clone(),
                    io::Error::other(error),
                )
            })?
            .as_secs();
        let result = with_refresh_lock(
            RefreshLockRequest {
                cache_root: &self.root,
                lock_path: &self.lock_path,
                target_path: &self.path,
                network: MAINNET_NETWORK,
                now_unix_secs: now,
                lock_stale_after_seconds: 600,
            },
            operation,
            || self.publish_under_lock(endpoint, prefix, checkpoint.version, pages, now),
        );
        match result {
            Err(HostCacheError::Operation {
                source: CacheFileError::RefreshAlreadyInProgress { .. },
                ..
            }) => Ok(HistoryCacheObservation::skipped(
                checkpoint.version,
                "another history writer owns the lock",
            )),
            result => result,
        }
    }

    fn publish_under_lock(
        &self,
        endpoint: &str,
        prefix: &str,
        version: u64,
        pages: &[RegistryHistoryPage],
        now_unix_secs: u64,
    ) -> Result<HistoryCacheObservation, HostCacheError> {
        let (mut document, _) = self.read()?;
        let mut existing = document
            .checkpoints
            .iter()
            .position(|entry| entry.endpoint == endpoint && entry.prefix == prefix);
        if let Some(index) = existing
            && document.checkpoints[index].version() >= version
            && RegistryKeyFamilyCheckpoint::replay(
                prefix,
                document.checkpoints[index].pages.clone(),
            )
            .is_err()
        {
            document.checkpoints.remove(index);
            existing = None;
        }
        if let Some(index) = existing {
            if document.checkpoints[index].version() >= version {
                return Ok(HistoryCacheObservation::skipped(
                    version,
                    "an equal or newer prefix is already published",
                ));
            }
        } else if document.checkpoints.len() == MAX_HISTORY_CHECKPOINTS {
            return Ok(HistoryCacheObservation::skipped(
                version,
                "checkpoint identity retention limit reached",
            ));
        }
        let entry = HistoryEntry {
            network: MAINNET_NETWORK.to_string(),
            registry_canister: MAINNET_REGISTRY_CANISTER_ID.to_string(),
            endpoint: endpoint.to_string(),
            prefix: prefix.to_string(),
            pages: pages.to_vec(),
        };
        if let Some(index) = existing {
            document.checkpoints[index] = entry;
        } else {
            document.checkpoints.push(entry);
        }
        document.checkpoints.sort_by(|left, right| {
            (&left.endpoint, &left.prefix).cmp(&(&right.endpoint, &right.prefix))
        });
        document.fetched_at = format_utc_timestamp_secs(now_unix_secs);
        document.digest = "0".repeat(64);
        if canonical_json_serialized_len(&document).map_err(|error| self.serialize_error(error))?
            > MAX_HISTORY_BYTES
        {
            return Ok(HistoryCacheObservation::skipped(
                version,
                "history file byte retention limit reached",
            ));
        }
        document.digest = history_digest(&document).map_err(|error| self.serialize_error(error))?;
        write_managed_file_atomically(&self.root, &self.path, |file| {
            serde_json::to_writer(file, &document)
        })
        .map_err(operation)?;
        Ok(HistoryCacheObservation {
            disposition: RegistryHistoryCacheDisposition::Published,
            through_version: version,
            reason: None,
        })
    }

    fn serialize_error(&self, source: serde_json::Error) -> HostCacheError {
        HostCacheError::serialize_cache(COMPONENT, self.path.clone(), source)
    }

    fn read(&self) -> Result<(HistoryDocument, Option<String>), HostCacheError> {
        let bytes = match read_bounded_managed_file(&self.root, &self.path, MAX_HISTORY_BYTES, None)
        {
            Ok(Some(bytes)) => bytes,
            Ok(None) => return Ok((HistoryDocument::empty(), None)),
            Err(BoundedManagedFileReadError::Operation(error)) => return Err(operation(error)),
            Err(BoundedManagedFileReadError::Read { path, source }) => {
                return Err(HostCacheError::read_cache(COMPONENT, path, source));
            }
            Err(
                BoundedManagedFileReadError::LimitExceeded { .. }
                | BoundedManagedFileReadError::Accounting { .. },
            ) => {
                return Ok((
                    HistoryDocument::empty(),
                    Some("history file exceeds its byte ceiling".to_string()),
                ));
            }
        };
        let document = match serde_json::from_slice::<HistoryDocument>(&bytes) {
            Ok(document) => document,
            Err(error) => return Ok((HistoryDocument::empty(), Some(error.to_string()))),
        };
        if let Err(reason) = document.validate() {
            return Ok((HistoryDocument::empty(), Some(reason)));
        }
        Ok((document, None))
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct HistoryDocument {
    schema_version: u32,
    network: String,
    fetched_at: String,
    checkpoints: Vec<HistoryEntry>,
    digest: String,
}

impl HistoryDocument {
    fn empty() -> Self {
        Self {
            schema_version: REGISTRY_HISTORY_SCHEMA_VERSION,
            network: MAINNET_NETWORK.to_string(),
            fetched_at: String::new(),
            checkpoints: Vec::new(),
            digest: String::new(),
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.schema_version != REGISTRY_HISTORY_SCHEMA_VERSION {
            return Err("unsupported Registry history schema".to_string());
        }
        if self.network != MAINNET_NETWORK || parse_utc_timestamp_secs(&self.fetched_at).is_none() {
            return Err("invalid history network or publication timestamp".to_string());
        }
        if self.checkpoints.len() > MAX_HISTORY_CHECKPOINTS {
            return Err("too many history checkpoint identities".to_string());
        }
        let mut identities = BTreeSet::new();
        for entry in &self.checkpoints {
            if entry.network != MAINNET_NETWORK
                || entry.registry_canister != MAINNET_REGISTRY_CANISTER_ID
                || entry.endpoint.is_empty()
                || entry.endpoint.len() > 2_048
                || entry.prefix.is_empty()
                || entry.prefix.len() > 1_024
                || entry.pages.is_empty()
                || entry.pages.len() > MAX_HISTORY_PAGES
                || !identities.insert((&entry.endpoint, &entry.prefix))
            {
                return Err(
                    "invalid or duplicate history checkpoint identity or page count".to_string(),
                );
            }
        }
        let digest = history_digest(self).map_err(|error| error.to_string())?;
        if self.digest != digest {
            return Err("Registry history checksum mismatch".to_string());
        }
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct HistoryEntry {
    network: String,
    registry_canister: String,
    endpoint: String,
    prefix: String,
    pages: Vec<RegistryHistoryPage>,
}

impl HistoryEntry {
    fn version(&self) -> u64 {
        self.pages.last().map_or(0, |page| page.through_version)
    }
}

const fn operation(source: CacheFileError) -> HostCacheError {
    HostCacheError::operation(COMPONENT, source)
}

fn history_digest(document: &HistoryDocument) -> Result<String, serde_json::Error> {
    canonical_json_sha256(&(
        document.schema_version,
        &document.network,
        &document.fetched_at,
        &document.checkpoints,
    ))
    .map(|digest| hex_bytes(&digest))
}

#[cfg(test)]
mod tests;
