//! Module: ic::node_status
//!
//! Responsibility: observed Dashboard node-status snapshots, caches, projections, and text.
//! Does not own: HTTP transport, Registry authority, command parsing, or process output.
//! Boundary: derives node, Subnet, and provider views from one finite off-chain snapshot.

#[cfg(feature = "host")]
mod cache;
mod counts;
mod model;
mod projection;
mod text;
mod validation;

#[cfg(feature = "host")]
pub use cache::{
    build_ic_node_provider_status_report, build_ic_node_provider_status_report_with_source,
    build_ic_node_status_report, build_ic_node_status_report_with_source,
    build_ic_subnet_status_report, build_ic_subnet_status_report_with_source,
    ic_node_status_cache_path, ic_node_status_refresh_lock_path,
    load_cached_ic_node_status_snapshot, load_or_refresh_missing_ic_node_status_snapshot,
    load_or_refresh_missing_ic_node_status_snapshot_with_source,
    load_or_refresh_stale_ic_node_status_snapshot,
    load_or_refresh_stale_ic_node_status_snapshot_with_source, refresh_ic_node_status_snapshot,
    refresh_ic_node_status_snapshot_with_source,
};
pub(in crate::ic) use counts::{node_status_counts, node_status_group_counts};
#[cfg(feature = "host")]
pub use model::IcNodeStatusSourceData;
pub use model::{
    IcNodeAssignmentStatusCounts, IcNodeCountComparison, IcNodeCountComparisonCounts,
    IcNodeOperationalStatus, IcNodeProviderStatusReport, IcNodeProviderStatusRow,
    IcNodeStatusCacheEvidence, IcNodeStatusCounts, IcNodeStatusGroupCounts,
    IcNodeStatusObservation, IcNodeStatusProjectionError, IcNodeStatusReport, IcNodeStatusRow,
    IcNodeStatusScope, IcNodeStatusSnapshot, IcNodeStatusSnapshotRequest, IcNodeStatusView,
    IcSubnetStatusReport, IcSubnetStatusRow,
};
#[cfg(feature = "host")]
pub use model::{
    IcNodeStatusCacheRequest, IcNodeStatusHostError, IcNodeStatusReadRequest,
    IcNodeStatusRefreshReport, IcNodeStatusRefreshRequest,
};
pub use projection::{
    ic_node_provider_status_report_from_snapshot, ic_node_status_report_from_snapshot,
    ic_subnet_status_report_from_snapshot,
};
#[cfg(feature = "host")]
pub use text::ic_node_status_refresh_report_text;
pub use text::{
    ic_node_provider_status_report_text, ic_node_status_report_text, ic_subnet_status_report_text,
};
#[cfg(feature = "host")]
pub(in crate::ic) use validation::canonicalize_node_status_rows;
pub(in crate::ic) use validation::{
    validate_canonical_node_status_rows, validate_default_node_scope,
};

/// Current schema version for observed node-status reports and caches.
pub const IC_NODE_STATUS_SCHEMA_VERSION: u32 = 1;

/// Maximum rows accepted from the finite Dashboard node resource.
pub const MAX_IC_NODE_STATUS_ROWS: u32 = 10_000;

/// Age after which an observed node-status cache refreshes automatically.
pub const DEFAULT_IC_NODE_STATUS_STALE_AFTER_SECONDS: u64 = 60;

/// Age after which an abandoned node-status refresh lock is stale.
pub const DEFAULT_IC_NODE_STATUS_REFRESH_LOCK_STALE_SECONDS: u64 = 5 * 60;

pub(super) const IC_NODE_STATUS_SCOPE: &str = "dashboard_mainnet_default";

#[cfg(test)]
mod tests;
