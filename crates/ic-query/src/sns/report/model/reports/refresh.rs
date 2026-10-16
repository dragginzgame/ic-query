//! Module: sns::report::model::reports::refresh
//!
//! Responsibility: describe one published complete SNS collection snapshot.
//! Does not own: page fetching, persistence, or text rendering.
//! Boundary: collection identity and row accounting share one schema-1 result.

use serde::Serialize;

///
/// SnsSnapshotRefreshReport
///
/// Publication result shared by complete neuron and proposal snapshot refreshes.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsSnapshotRefreshReport {
    /// Serialized report schema version, currently 1.
    pub schema_version: u32,
    /// Network whose snapshot was collected.
    pub network: String,
    /// SNS-W canister used to resolve the SNS.
    pub sns_wasm_canister_id: String,
    /// UTC collection timestamp.
    pub fetched_at: String,
    /// Explicit replica endpoint used for collection.
    pub source_endpoint: String,
    /// Collector identity.
    pub fetched_by: String,
    /// SNS id resolved for this refresh.
    pub id: usize,
    /// SNS display name.
    pub name: String,
    /// Root principal identifying the SNS.
    pub root_canister_id: String,
    /// Governance principal queried for rows.
    pub governance_canister_id: String,
    /// Published collection: neurons or proposals.
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
