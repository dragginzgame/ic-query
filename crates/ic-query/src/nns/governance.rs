//! Module: nns::governance
//!
//! Responsibility: shared host contracts for complete NNS Governance collections.
//! Does not own: proposal or neuron paging, cache paths, or report rendering.
//! Boundary: centralizes identical refresh, cache-scope, attempt, and provenance DTOs.

use crate::{
    ic_registry::MAINNET_GOVERNANCE_CANISTER_ID,
    snapshot_cache::{SnapshotRefreshAttempt, SnapshotRefreshProgress},
};
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::path::{Path, PathBuf};

/// Refresh-attempt fields owned by NNS Governance collection metadata.
pub(super) const NNS_GOVERNANCE_ATTEMPT_METADATA_FIELDS: &[&str] = &["governance_canister_id"];

///
/// NnsGovernanceRefreshRequest
///
/// Shared request for one complete NNS Governance collection refresh.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsGovernanceRefreshRequest {
    /// Root directory containing shared caches.
    pub cache_root: PathBuf,
    /// Requested network identity.
    pub network: String,
    /// Replica endpoint used for every page.
    pub source_endpoint: String,
    /// Caller-provided collection time in Unix seconds.
    pub now_unix_secs: u64,
    /// Governance page size.
    pub page_size: u32,
    /// Optional diagnostic page cap.
    pub max_pages: Option<u32>,
}

impl NnsGovernanceRefreshRequest {
    /// Construct a complete NNS Governance collection refresh request.
    #[must_use]
    pub fn new(
        cache_root: impl Into<PathBuf>,
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        page_size: u32,
    ) -> Self {
        Self {
            cache_root: cache_root.into(),
            network: network.into(),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            page_size,
            max_pages: None,
        }
    }

    /// Stop diagnostically before fetching more than the given pages.
    #[must_use]
    pub const fn with_max_pages(mut self, max_pages: Option<u32>) -> Self {
        self.max_pages = max_pages;
        self
    }
}

///
/// NnsGovernanceCacheRequest
///
/// Shared local cache scope for complete NNS Governance collections.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NnsGovernanceCacheRequest {
    /// Root directory containing shared caches.
    pub cache_root: PathBuf,
    /// Requested network identity.
    pub network: String,
}

impl NnsGovernanceCacheRequest {
    /// Construct a local NNS Governance cache request.
    #[must_use]
    pub fn new(cache_root: impl Into<PathBuf>, network: impl Into<String>) -> Self {
        Self {
            cache_root: cache_root.into(),
            network: network.into(),
        }
    }

    /// Return the shared cache root.
    #[must_use]
    pub fn cache_root(&self) -> &Path {
        &self.cache_root
    }
}

///
/// NnsGovernanceRefreshAttemptStatus
///
/// Serializable lifecycle evidence for an NNS Governance collection refresh.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NnsGovernanceRefreshAttemptStatus {
    /// Attempt lifecycle status.
    pub status: String,
    /// Attempt start timestamp.
    pub started_at: String,
    /// Latest attempt update timestamp.
    pub updated_at: String,
    /// Requested page size.
    pub page_size: u32,
    /// Successfully retained pages.
    pub pages_fetched: u32,
    /// Successfully retained rows.
    pub rows_fetched: usize,
    /// Latest exclusive cursor.
    pub last_cursor: Option<String>,
    /// Latest terminal error.
    pub last_error: Option<String>,
}

///
/// NnsGovernanceCacheMetadata
///
/// Shared flattened provenance for an NNS Governance snapshot or attempt.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub(super) struct NnsGovernanceCacheMetadata {
    /// NNS Governance canister principal.
    pub(super) governance_canister_id: String,
}

/// Construct canonical mainnet NNS Governance cache metadata.
#[must_use]
pub(super) fn mainnet_governance_cache_metadata() -> NnsGovernanceCacheMetadata {
    NnsGovernanceCacheMetadata {
        governance_canister_id: MAINNET_GOVERNANCE_CANISTER_ID.to_string(),
    }
}

/// Validate the Governance canister identity in shared cache metadata.
pub(super) fn validate_governance_cache_metadata(
    metadata: &NnsGovernanceCacheMetadata,
) -> Result<(), String> {
    if metadata.governance_canister_id == MAINNET_GOVERNANCE_CANISTER_ID {
        return Ok(());
    }
    Err(format!(
        "governance_canister_id is {}, expected {MAINNET_GOVERNANCE_CANISTER_ID}",
        metadata.governance_canister_id
    ))
}

/// Project a stored refresh attempt into its report-safe lifecycle fields.
#[must_use]
pub(super) fn governance_refresh_attempt_status<Metadata>(
    attempt: SnapshotRefreshAttempt<Metadata>,
) -> NnsGovernanceRefreshAttemptStatus {
    NnsGovernanceRefreshAttemptStatus {
        status: attempt.status,
        started_at: attempt.started_at,
        updated_at: attempt.updated_at,
        page_size: attempt.page_size,
        pages_fetched: attempt.pages_fetched,
        rows_fetched: attempt.rows_fetched,
        last_cursor: attempt.last_cursor,
        last_error: attempt.last_error,
    }
}

/// Recover collection progress from one stored NNS Governance attempt.
#[must_use]
pub(super) fn governance_refresh_progress<Metadata>(
    attempt: SnapshotRefreshAttempt<Metadata>,
) -> SnapshotRefreshProgress {
    SnapshotRefreshProgress::new(
        attempt.pages_fetched,
        attempt.rows_fetched,
        attempt.last_cursor,
    )
}
