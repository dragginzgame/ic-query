//! Module: cache::model
//!
//! Responsibility: define shared cache-validation and refresh-attempt contracts.
//! Does not own: filesystem traversal, cache-family validation, or CLI output.
//! Boundary: preserves typed family validation and refresh-attempt evidence.

use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::fmt;

///
/// CacheValidationStatus
///
/// Semantic validation result for an existing family-specific cache.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheValidationStatus {
    /// The cache passed its family-specific schema, identity, and completeness checks.
    #[serde(rename = "ok")]
    Valid,
    /// The cache exists but failed family-specific validation.
    Invalid,
}

impl CacheValidationStatus {
    /// Return the stable serialized status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "ok",
            Self::Invalid => "invalid",
        }
    }
}

impl fmt::Display for CacheValidationStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

///
/// CacheRefreshAttemptStatus
///
/// Lifecycle state for a complete-cache refresh attempt.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheRefreshAttemptStatus {
    /// A refresh started or published intermediate collection progress.
    Running,
    /// A refresh exhausted its source and published a complete cache.
    Complete,
    /// A refresh terminated without replacing the complete cache.
    Failed,
}

impl CacheRefreshAttemptStatus {
    /// Return the stable serialized lifecycle label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Complete => "complete",
            Self::Failed => "failed",
        }
    }
}

impl fmt::Display for CacheRefreshAttemptStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests;

///
/// SnapshotRefreshAttemptStatus
///
/// Report-safe lifecycle evidence projected after a family validates its attempt sidecar.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnapshotRefreshAttemptStatus {
    /// Current lifecycle status.
    pub status: CacheRefreshAttemptStatus,
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
    /// Terminal failure evidence.
    pub last_error: Option<String>,
}
