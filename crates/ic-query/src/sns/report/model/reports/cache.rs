//! Module: sns::report::model::reports::cache
//!
//! Responsibility: shared SNS complete-snapshot cache inspection DTOs.
//! Does not own: cache discovery, collection-specific storage, or rendering.
//! Boundary: preserves fields common to neuron cache list/status reports.

use super::SnsRefreshAttemptStatus;
use crate::cache::CacheValidationStatus;
use serde::Serialize;

///
/// SnsCacheListReport
///
/// Serializable report listing one family of complete local SNS caches.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsCacheListReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub caches: Vec<SnsCacheSummary>,
}

///
/// SnsCacheStatusReport
///
/// Serializable report describing one explicitly addressed SNS cache.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsCacheStatusReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_root: String,
    pub input: String,
    pub found: bool,
    pub cache: Option<SnsCacheSummary>,
    pub expected_cache_path: String,
    /// Independent refresh-attempt sidecar path.
    pub refresh_attempt_path: String,
    /// Latest admitted attempt evidence, independent of snapshot validity.
    pub latest_attempt: Option<SnsRefreshAttemptStatus>,
}

///
/// SnsCacheSummary
///
/// Serializable summary of one complete SNS snapshot cache.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsCacheSummary {
    /// Inspected snapshot path, including invalid files.
    pub cache_path: String,
    /// Independent refresh-attempt sidecar path.
    pub refresh_attempt_path: String,
    /// Latest admitted attempt evidence, independent of snapshot validity.
    pub latest_attempt: Option<SnsRefreshAttemptStatus>,
    /// Admitted complete facts or the original content refusal.
    #[serde(flatten)]
    pub admission: SnsCacheAdmission,
}

impl SnsCacheSummary {
    /// Return facts only for an admitted complete snapshot.
    #[must_use]
    pub const fn valid(&self) -> Option<&SnsValidCacheSummary> {
        match &self.admission {
            SnsCacheAdmission::Valid { data } => Some(data),
            SnsCacheAdmission::Invalid { .. } => None,
        }
    }

    /// Return the cache admission outcome.
    #[must_use]
    pub const fn cache_status(&self) -> CacheValidationStatus {
        match self.admission {
            SnsCacheAdmission::Valid { .. } => CacheValidationStatus::Valid,
            SnsCacheAdmission::Invalid { .. } => CacheValidationStatus::Invalid,
        }
    }

    /// Original diagnostic for invalid content.
    #[must_use]
    pub fn cache_error(&self) -> Option<&str> {
        match &self.admission {
            SnsCacheAdmission::Invalid { cache_error } => Some(cache_error),
            SnsCacheAdmission::Valid { .. } => None,
        }
    }
}

///
/// SnsCacheAdmission
///
/// Complete snapshot facts or the original admission failure, never fabricated metadata.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "cache_status", rename_all = "snake_case")]
pub enum SnsCacheAdmission {
    /// Admitted complete snapshot.
    Valid {
        /// Admitted complete collection facts.
        #[serde(flatten)]
        data: SnsValidCacheSummary,
    },
    /// Content failed admission; the envelope retains its path and independent attempt.
    Invalid {
        /// Original admission failure.
        cache_error: String,
    },
}

///
/// SnsValidCacheSummary
///
/// Identity and collection facts from an admitted complete SNS snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsValidCacheSummary {
    /// Observed SNS-W list position.

    /// Observed SNS name.

    /// Admitted Governance principal.

    /// Admitted Governance principal.
    pub governance_canister_id: String,
    /// Rows in the admitted complete collection.
    pub row_count: usize,
    /// Pages collected through API exhaustion.
    pub page_count: u32,
    /// Requested rows per source page.
    pub page_size: u32,
    /// Admitted collection timestamp.
    pub fetched_at: String,
    /// Explicit endpoint that supplied the admitted snapshot.
    pub source_endpoint: String,
}
