//! Module: sns::report::model::reports::upgrade
//!
//! Responsibility: bounded native SNS upgrade report DTOs.
//! Does not own: Governance or SNS-W calls, source validation, lookup, or rendering.
//! Boundary: preserves native deployed, pending, and next blessed SNS versions.

use serde::Serialize;

///
/// SnsVersion
///
/// Native six-role SNS Wasm version represented as lowercase hexadecimal hashes.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsVersion {
    /// Archive Wasm hash.
    pub archive_wasm_hash_hex: String,
    /// Root Wasm hash.
    pub root_wasm_hash_hex: String,
    /// Swap Wasm hash.
    pub swap_wasm_hash_hex: String,
    /// Ledger Wasm hash.
    pub ledger_wasm_hash_hex: String,
    /// Governance Wasm hash.
    pub governance_wasm_hash_hex: String,
    /// Ledger index Wasm hash.
    pub index_wasm_hash_hex: String,
}

///
/// SnsPendingUpgrade
///
/// Native pending-upgrade state reported by SNS Governance.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsPendingUpgrade {
    /// Native deadline after which Governance may mark the upgrade failed.
    pub mark_failed_at_seconds: u64,
    /// Native Governance upgrade-lock checking value.
    pub checking_upgrade_lock: u64,
    /// NNS proposal that initiated the pending upgrade.
    pub proposal_id: u64,
    /// Pending target version, when Governance returned it.
    pub target_version: Option<SnsVersion>,
}

///
/// SnsUpgradeQueryGap
///
/// Failed next-version query retained after deployed-version collection succeeded.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsUpgradeQueryGap {
    /// Native SNS-W method that failed.
    pub method: String,
    /// Transport, encoding, or decoding failure retained for diagnostics.
    pub reason: String,
}

///
/// SnsUpgradeReport
///
/// Bounded live report of one SNS deployed, pending, and next blessed version.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsUpgradeReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Requested IC network identity.
    pub network: String,
    /// Mainnet SNS-W canister used for discovery and next-version lookup.
    pub sns_wasm_canister_id: String,
    /// Collection timestamp in UTC.
    pub fetched_at: String,
    /// IC API endpoint used for all calls.
    pub source_endpoint: String,
    /// Collector identity recorded by the source request.
    pub fetched_by: String,
    /// SNS-W list id assigned to this deployed SNS.
    pub id: usize,
    /// SNS name resolved during discovery.
    pub name: String,
    /// Root canister identity used to resolve this SNS.
    pub root_canister_id: String,
    /// Governance canister queried for deployed and pending versions.
    pub governance_canister_id: String,
    /// Native Governance running-version method.
    pub running_version_method: String,
    /// Native SNS-W next-version method.
    pub next_version_method: String,
    /// Whether both component responses represent one authoritative point in time.
    pub point_in_time_guaranteed: bool,
    /// Fixed number of bounded upgrade component queries attempted.
    pub component_query_count: usize,
    /// Number of component queries that returned successfully.
    pub successful_component_query_count: usize,
    /// Number of retained next-version query gaps.
    pub component_gap_count: usize,
    /// Governance-reported deployed SNS version.
    pub deployed_version: SnsVersion,
    /// Governance-reported pending upgrade, when present.
    pub pending_upgrade: Option<SnsPendingUpgrade>,
    /// Next blessed SNS-W version, or `None` when no successor exists.
    pub next_version: Option<SnsVersion>,
    /// Failed next-version query, distinct from a successful response with no successor.
    pub next_version_gap: Option<SnsUpgradeQueryGap>,
}
