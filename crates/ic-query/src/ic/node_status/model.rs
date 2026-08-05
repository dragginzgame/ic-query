//! Module: ic::node_status::model
//!
//! Responsibility: public observed node-status requests, evidence, reports, and errors.
//! Does not own: source calls, cache IO, projection, or rendering.
//! Boundary: preserves raw Dashboard status data and explicit off-chain/cache provenance.

use crate::ic::IcDashboardReportProvenance;
use serde::{Deserialize as SerdeDeserialize, Serialize};
use std::fmt;
#[cfg(feature = "host")]
use std::path::PathBuf;
use thiserror::Error as ThisError;

///
/// IcNodeStatusScope
///
/// Dashboard collection scope retained by an observed node-status snapshot.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IcNodeStatusScope {
    /// Dashboard's default public-mainnet node set, excluding cloud-engine Type4 nodes.
    DashboardMainnetDefault,
}

impl IcNodeStatusScope {
    /// Return the stable serialized scope label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DashboardMainnetDefault => super::IC_NODE_STATUS_SCOPE,
        }
    }
}

///
/// IcNodeOperationalStatus
///
/// Known Dashboard node-status classification used for filtering and counts.
///

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IcNodeOperationalStatus {
    /// The Dashboard reports the node as up.
    Up,
    /// The Dashboard reports the node as down.
    Down,
    /// The Dashboard reports the node as administratively disabled.
    Disabled,
    /// The Dashboard reports degraded operation.
    Degraded,
    /// The raw Dashboard value is not recognized by this release.
    Unknown,
}

impl IcNodeOperationalStatus {
    /// Classify raw Dashboard status text without discarding unknown future values.
    #[must_use]
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "UP" => Self::Up,
            "DOWN" => Self::Down,
            "DISABLED" => Self::Disabled,
            "DEGRADED" => Self::Degraded,
            _ => Self::Unknown,
        }
    }

    /// Return the stable lowercase display label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Disabled => "disabled",
            Self::Degraded => "degraded",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for IcNodeOperationalStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

///
/// IcNodeStatusCounts
///
/// Exact status totals derived from raw observed node rows.
///

#[derive(Clone, Debug, Default, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct IcNodeStatusCounts {
    /// Total observed rows in the group.
    pub total: usize,
    /// Rows classified as `UP`.
    pub up: usize,
    /// Rows classified as `DOWN`.
    pub down: usize,
    /// Rows classified as `DISABLED`.
    pub disabled: usize,
    /// Rows classified as `DEGRADED`.
    pub degraded: usize,
    /// Rows whose raw status is not recognized.
    pub unknown: usize,
}

impl IcNodeStatusCounts {
    /// Return every row not classified as raw Dashboard `UP`.
    #[must_use]
    pub const fn non_up(&self) -> usize {
        self.total.saturating_sub(self.up)
    }
}

///
/// IcNodeAssignmentStatusCounts
///
/// Operational-status totals partitioned by mutually exclusive assignment class.
///

#[derive(Clone, Debug, Default, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct IcNodeAssignmentStatusCounts {
    /// Status totals for nodes with an observed assigned Subnet.
    pub assigned: IcNodeStatusCounts,
    /// Status totals for rows whose raw node type is `UNASSIGNED`.
    pub unassigned: IcNodeStatusCounts,
    /// Status totals for rows whose raw node type is `API_BOUNDARY`.
    pub api_boundary: IcNodeStatusCounts,
    /// Status totals for rows without a known assignment class.
    pub unknown: IcNodeStatusCounts,
}

///
/// IcNodeStatusGroupCounts
///
/// Overall and assignment-partitioned status totals for one aggregate group.
///

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct IcNodeStatusGroupCounts {
    /// Raw operational-status totals for the group.
    pub statuses: IcNodeStatusCounts,
    /// Raw operational-status totals partitioned by assignment class.
    pub assignment_statuses: IcNodeAssignmentStatusCounts,
}

///
/// IcNodeStatusRow
///
/// One raw node observation retained from the official Dashboard node resource.
///

#[derive(Clone, Debug, Eq, PartialEq, SerdeDeserialize, Serialize)]
pub struct IcNodeStatusRow {
    /// Canonical node principal.
    pub node_id: String,
    /// Canonical node-operator principal.
    pub node_operator_id: String,
    /// Canonical node-provider principal.
    pub node_provider_id: String,
    /// Off-chain Dashboard node-provider name.
    pub node_provider_name: String,
    /// Raw Dashboard node type.
    pub node_type: String,
    /// Raw Registry reward type reported by the Dashboard.
    pub node_reward_type: String,
    /// Raw Dashboard operational status.
    pub status: String,
    /// Raw Dashboard alert name when present.
    pub alert_name: Option<String>,
    /// Assigned Subnet principal when present.
    pub subnet_id: Option<String>,
    /// Cloud-engine Subnet principal when present in a future compatible scope.
    pub cloud_engine_subnet_id: Option<String>,
    /// Dashboard data-center identifier.
    pub data_center_id: String,
    /// Dashboard data-center name.
    pub data_center_name: String,
    /// Dashboard infrastructure owner label.
    pub owner: String,
    /// Raw Dashboard geographic region label.
    pub region: String,
    /// Observed GuestOS version when reported.
    pub guestos_version: Option<String>,
    /// Observed GuestOS trusted-execution status when reported.
    pub guestos_tee_active: Option<bool>,
    /// Observed node IP address when reported.
    pub ip_address: Option<String>,
    /// Observed IPv4-connectivity state when reported.
    pub ipv4_connectivity_status: Option<bool>,
    /// Dashboard hardware-generation label when reported.
    pub node_hardware_generation: Option<String>,
}

impl IcNodeStatusRow {
    /// Return the known classification of this row's raw status.
    #[must_use]
    pub fn operational_status(&self) -> IcNodeOperationalStatus {
        IcNodeOperationalStatus::from_raw(&self.status)
    }

    /// Return whether the raw row is anything other than known `UP`.
    #[must_use]
    pub fn is_non_up(&self) -> bool {
        self.operational_status() != IcNodeOperationalStatus::Up
    }
}

///
/// IcNodeStatusCacheEvidence
///
/// Caller-relative local cache evidence attached to a projected status report.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcNodeStatusCacheEvidence {
    /// Canonical local cache path used by the report.
    pub cache_path: String,
    /// Whether the cache is within the default age policy for this caller's time.
    pub cache_fresh: bool,
    /// Cache age at report construction.
    pub age_seconds: u64,
    /// Age threshold used to classify freshness.
    pub stale_after_seconds: u64,
}

///
/// IcNodeStatusObservation
///
/// Shared source, scope, and optional cache evidence for status reports.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcNodeStatusObservation {
    #[serde(flatten)]
    /// Official Dashboard source provenance.
    pub source: IcDashboardReportProvenance,
    /// Explicit node collection scope.
    pub scope: IcNodeStatusScope,
    /// Whether cloud-engine nodes were included in the source collection.
    pub cloud_engine_nodes_included: bool,
    /// Local cache evidence, absent for a pure live snapshot.
    pub cache: Option<IcNodeStatusCacheEvidence>,
}

///
/// IcNodeStatusSnapshot
///
/// Complete canonical observed node snapshot used by every status projection.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcNodeStatusSnapshot {
    #[serde(flatten)]
    /// Shared source and scope evidence.
    pub observation: IcNodeStatusObservation,
    /// Exact number of raw node rows.
    pub node_count: usize,
    /// Snapshot-wide overall and assignment-partitioned totals.
    pub counts: IcNodeStatusGroupCounts,
    /// Canonically ordered raw node rows.
    pub nodes: Vec<IcNodeStatusRow>,
}

///
/// IcNodeStatusSnapshotRequest
///
/// Request for one live finite Dashboard node-status snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcNodeStatusSnapshotRequest {
    /// Dashboard API v3 base endpoint.
    pub source_endpoint: String,
    /// Caller-supplied collection timestamp as Unix seconds.
    pub now_unix_secs: u64,
}

impl IcNodeStatusSnapshotRequest {
    /// Construct one live node-status snapshot request.
    #[must_use]
    pub fn new(source_endpoint: impl Into<String>, now_unix_secs: u64) -> Self {
        Self {
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
        }
    }
}

///
/// IcNodeStatusSourceData
///
/// Untrusted raw node rows and provenance returned by a Dashboard source capability.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcNodeStatusSourceData {
    /// Source-call provenance echoed by the source.
    pub source: crate::ic::IcSourceRequest,
    /// Explicit Dashboard collection scope.
    pub scope: IcNodeStatusScope,
    /// Whether the source claims cloud-engine node inclusion.
    pub cloud_engine_nodes_included: bool,
    /// Raw observed node rows.
    pub nodes: Vec<IcNodeStatusRow>,
}

///
/// IcNodeStatusView
///
/// Target and attention selection shared by node, Subnet, and provider views.
///

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct IcNodeStatusView {
    /// Optional exact identifier or unique principal prefix.
    pub target: Option<String>,
    /// Whether fully-up rows or groups are included without a target.
    pub include_all: bool,
}

impl IcNodeStatusView {
    /// Construct the default attention-only view.
    #[must_use]
    pub const fn attention() -> Self {
        Self {
            target: None,
            include_all: false,
        }
    }

    /// Select one exact identifier or unique principal prefix.
    #[must_use]
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Include fully-up rows as well as attention rows.
    #[must_use]
    pub const fn with_all(mut self, include_all: bool) -> Self {
        self.include_all = include_all;
        self
    }
}

///
/// IcNodeStatusReport
///
/// Node-level view over one complete observed status snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcNodeStatusReport {
    #[serde(flatten)]
    /// Shared source, scope, and cache evidence.
    pub observation: IcNodeStatusObservation,
    /// Total rows in the complete source snapshot.
    pub snapshot_node_count: usize,
    /// Snapshot-wide overall and assignment-partitioned totals.
    pub counts: IcNodeStatusGroupCounts,
    /// Whether the view includes fully-up rows.
    pub include_all: bool,
    /// Target text supplied by the caller.
    pub requested_target: Option<String>,
    /// Canonical target resolved by exact or unique-prefix matching.
    pub resolved_target: Option<String>,
    /// Stable label describing how the target resolved.
    pub resolved_from: Option<String>,
    /// Number of returned node rows.
    pub returned_node_count: usize,
    /// Selected canonical node rows.
    pub nodes: Vec<IcNodeStatusRow>,
}

///
/// IcSubnetStatusRow
///
/// Observed operational counts and conservative threshold evidence for one Subnet.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcSubnetStatusRow {
    /// Canonical Subnet principal.
    pub subnet_id: String,
    /// Operational-status totals for observed assigned nodes.
    pub statuses: IcNodeStatusCounts,
    /// Byzantine fault count `floor((n - 1) / 3)` for the observed membership size.
    pub fault_tolerance_node_count: usize,
    /// Additional down nodes required for the down count to exceed the derived threshold.
    pub additional_down_nodes_to_exceed_fault_tolerance: usize,
    /// Additional non-up nodes required for the conservative count to exceed the threshold.
    pub additional_non_up_nodes_to_exceed_fault_tolerance: usize,
    /// Whether the observed down count already exceeds the derived threshold.
    pub down_fault_tolerance_exceeded: bool,
    /// Whether the observed conservative non-up count already exceeds the threshold.
    pub conservative_non_up_fault_tolerance_exceeded: bool,
    /// Raw non-up node evidence in canonical node order.
    pub non_up_nodes: Vec<IcNodeStatusRow>,
}

///
/// IcSubnetStatusReport
///
/// Subnet-level operational view over one complete observed node snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcSubnetStatusReport {
    #[serde(flatten)]
    /// Shared source, scope, and cache evidence.
    pub observation: IcNodeStatusObservation,
    /// Total rows in the complete source snapshot.
    pub snapshot_node_count: usize,
    /// Snapshot rows with an assigned Subnet.
    pub assigned_node_count: usize,
    /// Number of observed Subnets before view filtering.
    pub subnet_count: usize,
    /// Number of observed Subnets containing a non-up node.
    pub attention_subnet_count: usize,
    /// Whether fully-up Subnets are included without a target.
    pub include_all: bool,
    /// Target text supplied by the caller.
    pub requested_target: Option<String>,
    /// Canonical Subnet target resolved by exact or unique-prefix matching.
    pub resolved_target: Option<String>,
    /// Stable label describing how the target resolved.
    pub resolved_from: Option<String>,
    /// Number of returned Subnet rows.
    pub returned_subnet_count: usize,
    /// Canonically ordered selected Subnet rows.
    pub subnets: Vec<IcSubnetStatusRow>,
}

///
/// IcNodeProviderStatusRow
///
/// Assignment and raw operational counts for one observed node provider.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcNodeProviderStatusRow {
    /// Canonical node-provider principal.
    pub node_provider_id: String,
    /// Off-chain Dashboard provider name.
    pub node_provider_name: String,
    /// Operational-status and assignment totals for the provider.
    pub counts: IcNodeStatusGroupCounts,
    /// Raw non-up node evidence in canonical node order.
    pub non_up_nodes: Vec<IcNodeStatusRow>,
}

///
/// IcNodeProviderStatusReport
///
/// Node-provider operational view over one complete observed node snapshot.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcNodeProviderStatusReport {
    #[serde(flatten)]
    /// Shared source, scope, and cache evidence.
    pub observation: IcNodeStatusObservation,
    /// Total rows in the complete source snapshot.
    pub snapshot_node_count: usize,
    /// Number of observed provider principals before view filtering.
    pub provider_count: usize,
    /// Number of provider groups containing a non-up node.
    pub attention_provider_count: usize,
    /// Whether fully-up providers are included without a target.
    pub include_all: bool,
    /// Target text supplied by the caller.
    pub requested_target: Option<String>,
    /// Canonical provider target resolved by exact or unique-prefix matching.
    pub resolved_target: Option<String>,
    /// Stable label describing how the target resolved.
    pub resolved_from: Option<String>,
    /// Number of returned provider rows.
    pub returned_provider_count: usize,
    /// Canonically ordered selected provider rows.
    pub providers: Vec<IcNodeProviderStatusRow>,
}

///
/// IcNodeStatusProjectionError
///
/// Failure to resolve or project a requested observed status view.
///

#[derive(Clone, Debug, Eq, PartialEq, ThisError)]
pub enum IcNodeStatusProjectionError {
    /// Snapshot-level counts or canonical ordering do not match raw rows.
    #[error("invalid observed node-status snapshot: {reason}")]
    InvalidSnapshot {
        /// Deterministic validation failure.
        reason: String,
    },
    /// The supplied target is empty after trimming.
    #[error("{kind} target must not be empty")]
    EmptyTarget {
        /// Projection kind being selected.
        kind: &'static str,
    },
    /// No observed identifier matches the supplied target.
    #[error("{kind} target {target:?} did not match the observed snapshot")]
    UnknownTarget {
        /// Projection kind being selected.
        kind: &'static str,
        /// Unmatched target text.
        target: String,
    },
    /// More than one observed identifier matches the supplied prefix.
    #[error("{kind} prefix {prefix:?} is ambiguous; matches: {matches:?}")]
    AmbiguousTarget {
        /// Projection kind being selected.
        kind: &'static str,
        /// Ambiguous principal prefix.
        prefix: String,
        /// Canonically ordered matching principals.
        matches: Vec<String>,
    },
}

///
/// IcNodeStatusCacheRequest
///
/// Stable identity of one network-level observed node-status cache.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcNodeStatusCacheRequest {
    /// Root directory containing all `ic-query` caches.
    pub cache_root: PathBuf,
    /// Requested network name.
    pub network: String,
}

#[cfg(feature = "host")]
impl IcNodeStatusCacheRequest {
    /// Construct one observed node-status cache identity.
    #[must_use]
    pub fn new(cache_root: impl Into<PathBuf>, network: impl Into<String>) -> Self {
        Self {
            cache_root: cache_root.into(),
            network: network.into(),
        }
    }
}

///
/// IcNodeStatusRefreshRequest
///
/// Settings for one forced observed node-status snapshot refresh.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcNodeStatusRefreshRequest {
    /// Stable cache identity.
    pub cache: IcNodeStatusCacheRequest,
    /// Explicit Dashboard API base endpoint.
    pub source_endpoint: String,
    /// Caller-supplied observation time as Unix seconds.
    pub now_unix_secs: u64,
    /// Age after which an abandoned refresh lock can be reclaimed.
    pub lock_stale_after_seconds: u64,
}

#[cfg(feature = "host")]
impl IcNodeStatusRefreshRequest {
    /// Construct one forced node-status snapshot refresh request.
    #[must_use]
    pub fn new(
        cache_root: impl Into<PathBuf>,
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
        lock_stale_after_seconds: u64,
    ) -> Self {
        Self {
            cache: IcNodeStatusCacheRequest::new(cache_root, network),
            source_endpoint: source_endpoint.into(),
            now_unix_secs,
            lock_stale_after_seconds,
        }
    }
}

///
/// IcNodeStatusReadRequest
///
/// Cache-backed status-view request shared by node, Subnet, and provider reports.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IcNodeStatusReadRequest {
    /// Live and cache settings used when refresh is required.
    pub refresh: IcNodeStatusRefreshRequest,
    /// Projection target and attention selection.
    pub view: IcNodeStatusView,
    /// Whether to force a live replacement before reading.
    pub force_refresh: bool,
}

#[cfg(feature = "host")]
impl IcNodeStatusReadRequest {
    /// Construct one stale-refresh status-view request.
    #[must_use]
    pub fn new(
        cache_root: impl Into<PathBuf>,
        network: impl Into<String>,
        source_endpoint: impl Into<String>,
        now_unix_secs: u64,
    ) -> Self {
        Self {
            refresh: IcNodeStatusRefreshRequest::new(
                cache_root,
                network,
                source_endpoint,
                now_unix_secs,
                super::DEFAULT_IC_NODE_STATUS_REFRESH_LOCK_STALE_SECONDS,
            ),
            view: IcNodeStatusView::attention(),
            force_refresh: false,
        }
    }

    /// Apply one target and attention selection.
    #[must_use]
    pub fn with_view(mut self, view: IcNodeStatusView) -> Self {
        self.view = view;
        self
    }

    /// Select a forced refresh before projecting the report.
    #[must_use]
    pub const fn with_force_refresh(mut self, force_refresh: bool) -> Self {
        self.force_refresh = force_refresh;
        self
    }
}

///
/// IcNodeStatusRefreshReport
///
/// Result of atomically replacing the complete observed node-status cache.
///

#[cfg(feature = "host")]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct IcNodeStatusRefreshReport {
    /// Cache/report schema version.
    pub schema_version: u32,
    /// Network identity attached to the replacement.
    pub network: String,
    /// Explicit Dashboard endpoint queried.
    pub source_endpoint: String,
    /// Canonical UTC collection timestamp.
    pub fetched_at: String,
    /// Collector identity.
    pub fetched_by: String,
    /// Published cache path.
    pub cache_path: String,
    /// Sibling refresh-lock path.
    pub refresh_lock_path: String,
    /// Whether an existing complete snapshot was replaced.
    pub replaced_existing_cache: bool,
    /// Number of published node rows.
    pub node_count: usize,
    /// Published overall and assignment-partitioned status totals.
    pub counts: IcNodeStatusGroupCounts,
}

///
/// IcNodeStatusHostError
///
/// Host source, cache, and projection failures for observed node-status reports.
///

#[cfg(feature = "host")]
#[derive(Debug, ThisError)]
pub enum IcNodeStatusHostError {
    /// The observed Dashboard source supports only public mainnet identity.
    #[error("observed IC node status supports only the mainnet `ic` network, not {network:?}")]
    UnsupportedNetwork {
        /// Unsupported requested network.
        network: String,
    },
    /// Live-source or source-validation failure.
    #[error(transparent)]
    Source(#[from] crate::ic::IcHostError),
    /// Pure projection failure.
    #[error(transparent)]
    Projection(#[from] IcNodeStatusProjectionError),
    /// Strict cache load found no complete snapshot.
    #[error("observed node-status cache is missing at {}", path.display())]
    MissingCache {
        /// Expected cache path.
        path: PathBuf,
    },
    /// Existing cache content could not be read.
    #[error("failed to read observed node-status cache at {}: {source}", path.display())]
    ReadCache {
        /// Cache path that failed.
        path: PathBuf,
        /// Underlying filesystem failure.
        source: std::io::Error,
    },
    /// Existing cache content was not valid JSON for the schema.
    #[error("failed to parse observed node-status cache at {}: {source}", path.display())]
    ParseCache {
        /// Cache path that failed.
        path: PathBuf,
        /// Underlying JSON failure.
        source: serde_json::Error,
    },
    /// Existing cache uses an unsupported schema version.
    #[error("observed node-status cache schema {version} is unsupported; expected {expected}")]
    UnsupportedCacheSchemaVersion {
        /// Observed schema version.
        version: u32,
        /// Required current schema version.
        expected: u32,
    },
    /// Existing cache belongs to another network.
    #[error("observed node-status cache network is {actual:?}, expected {requested:?}")]
    CacheNetworkMismatch {
        /// Requested network.
        requested: String,
        /// Network stored by the cache.
        actual: String,
    },
    /// Existing cache snapshot-key identity does not match its path.
    #[error(
        "observed node-status cache identity mismatch at {}: {field} is {actual:?}, expected {expected:?}",
        path.display()
    )]
    CacheIdentityMismatch {
        /// Cache path that failed validation.
        path: PathBuf,
        /// Identity field that differs.
        field: &'static str,
        /// Required identity value.
        expected: String,
        /// Stored identity value.
        actual: String,
    },
    /// Existing cache is structurally readable but semantically invalid.
    #[error("invalid observed node-status cache at {}: {reason}", path.display())]
    InvalidCache {
        /// Cache path that failed validation.
        path: PathBuf,
        /// Deterministic validation failure.
        reason: String,
    },
    /// A complete replacement could not be serialized.
    #[error("failed to serialize observed node-status cache at {}: {source}", path.display())]
    SerializeCache {
        /// Target cache path.
        path: PathBuf,
        /// Underlying JSON serialization failure.
        source: serde_json::Error,
    },
    /// Shared atomic cache or refresh-lock operation failed.
    #[error(transparent)]
    Cache(#[from] crate::cache_file::HostCacheError),
}
