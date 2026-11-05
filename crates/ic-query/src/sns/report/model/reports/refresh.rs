//! Module: sns::report::model::reports::refresh
//!
//! Responsibility: describe one published complete SNS collection snapshot.
//! Does not own: page fetching, persistence, or text rendering.
//! Boundary: collection identity and row accounting share one schema-1 result.

use crate::sns::report::SnsReportContext;
use serde::Serialize;

///
/// SnsSnapshotRefreshReport
///
/// Publication result shared by complete neuron snapshot refreshes.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsSnapshotRefreshReport {
    /// Serialized report schema version, currently 1.
    pub schema_version: u32,
    /// SNS identity and provenance, serialized as flat report fields.
    #[serde(flatten)]
    pub context: SnsReportContext,
    /// Published collection: neurons.
    pub collection: String,
    /// Path of the published complete snapshot.
    pub cache_path: String,
    /// Lock path used for publication.
    pub refresh_lock_path: String,
    /// Path of the refresh-attempt sidecar.
    pub refresh_attempt_path: String,
    /// Maximum requested rows per page.
    pub page_size: u32,
    /// Number of collected pages.
    pub page_count: u32,
    /// Number of rows in the complete snapshot.
    pub row_count: usize,
    /// Whether collection exhausted the API.
    pub complete: bool,
    /// Whether publication replaced an existing snapshot.
    pub replaced_existing_cache: bool,
    /// Whether the complete snapshot was published.
    pub wrote_cache: bool,
    /// Sidecar finalization failure after successful snapshot publication, if any.
    pub attempt_finalization_error: Option<String>,
}
