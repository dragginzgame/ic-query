//! Module: sns::report::model::reports::canisters
//!
//! Responsibility: SNS Root canister inventory and health report DTOs.
//! Does not own: Root transport, SNS lookup, report assembly, or rendering.
//! Boundary: preserves native canister roles, status, module hashes, and typed gaps.

use serde::{Deserialize, Serialize};

///
/// SnsCanisterCallType
///
/// Invocation mode used for a native SNS canister method.
///

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnsCanisterCallType {
    /// A non-replicated query call.
    Query,
    /// A non-replicated composite query call.
    CompositeQuery,
    /// A replicated update submitted through ingress.
    IngressUpdate,
}

impl SnsCanisterCallType {
    /// Return the stable JSON and text label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::CompositeQuery => "composite_query",
            Self::IngressUpdate => "ingress_update",
        }
    }
}

///
/// SnsCanisterRole
///
/// Native role assigned to a canister by the SNS Root interface.
///

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnsCanisterRole {
    /// SNS Root canister.
    Root,
    /// SNS Governance canister.
    Governance,
    /// SNS ledger canister.
    Ledger,
    /// SNS decentralization swap canister.
    Swap,
    /// SNS ledger index canister.
    Index,
    /// SNS ledger archive canister.
    Archive,
    /// Dapp canister registered with SNS Root.
    Dapp,
    /// SNS extension canister registered with SNS Root.
    Extension,
}

impl SnsCanisterRole {
    /// Return the native lowercase role label used in text reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Governance => "governance",
            Self::Ledger => "ledger",
            Self::Swap => "swap",
            Self::Index => "index",
            Self::Archive => "archive",
            Self::Dapp => "dapp",
            Self::Extension => "extension",
        }
    }
}

///
/// SnsCanisterStatus
///
/// Native running state returned by SNS Root for one canister.
///

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnsCanisterStatus {
    /// The canister is running.
    Running,
    /// The canister is stopping.
    Stopping,
    /// The canister is stopped.
    Stopped,
}

impl SnsCanisterStatus {
    /// Return the native lowercase canister-status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
        }
    }
}

///
/// SnsCanisterGapKind
///
/// Typed reason that Root inventory and health evidence could not be joined.
///

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SnsCanisterGapKind {
    /// The inventory response omitted a canister id for a native singleton role.
    InventoryCanisterIdMissing,
    /// The health response omitted the summary for an inventory canister.
    SummaryMissing,
    /// A health summary omitted its canister id.
    SummaryCanisterIdMissing,
    /// A singleton health summary identified a different canister than inventory.
    SummaryCanisterIdMismatch,
    /// A health summary identified a canister absent from inventory.
    SummaryNotInInventory,
    /// More than one health summary identified the same inventory canister and role.
    DuplicateSummary,
    /// A matched health summary omitted canister status.
    StatusMissing,
    /// The current Root health response does not expose this native role.
    HealthUnsupported,
}

impl SnsCanisterGapKind {
    /// Return the stable lowercase gap label used in text reports.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InventoryCanisterIdMissing => "inventory_canister_id_missing",
            Self::SummaryMissing => "summary_missing",
            Self::SummaryCanisterIdMissing => "summary_canister_id_missing",
            Self::SummaryCanisterIdMismatch => "summary_canister_id_mismatch",
            Self::SummaryNotInInventory => "summary_not_in_inventory",
            Self::DuplicateSummary => "duplicate_summary",
            Self::StatusMissing => "status_missing",
            Self::HealthUnsupported => "health_unsupported",
        }
    }
}

///
/// SnsCanisterGap
///
/// One explicit inventory or health relation gap returned by SNS Root.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsCanisterGap {
    /// Typed gap classification.
    pub kind: SnsCanisterGapKind,
    /// Native SNS canister role involved in the gap.
    pub role: SnsCanisterRole,
    /// Canister id supplied by the inventory response, when available.
    pub inventory_canister_id: Option<String>,
    /// Canister id supplied by the health summary, when available.
    pub summary_canister_id: Option<String>,
}

///
/// SnsCanisterRow
///
/// One canister in the authoritative SNS Root inventory.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsCanisterRow {
    /// Native SNS canister role.
    pub role: SnsCanisterRole,
    /// Canonical canister principal text.
    pub canister_id: String,
    /// Native canister running state when Root returned health evidence.
    pub status: Option<SnsCanisterStatus>,
    /// Running Wasm module hash as lowercase hexadecimal text.
    pub module_hash_hex: Option<String>,
    /// Raw cycle balance as unsigned decimal text.
    pub cycles: Option<String>,
    /// Raw memory size in bytes as unsigned decimal text.
    pub memory_size: Option<String>,
    /// Raw idle cycles burned per day as unsigned decimal text.
    pub idle_cycles_burned_per_day: Option<String>,
    /// Canonical controller principals returned by Root.
    pub controllers: Vec<String>,
}

///
/// SnsCanisterReport
///
/// Joined SNS Root inventory and operational-health report.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsCanisterReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Requested IC network identity.
    pub network: String,
    /// Mainnet SNS-W canister used to resolve the SNS.
    pub sns_wasm_canister_id: String,
    /// Collection timestamp in UTC.
    pub fetched_at: String,
    /// IC API endpoint used for SNS-W and Root calls.
    pub source_endpoint: String,
    /// Collector identity recorded by the source request.
    pub fetched_by: String,
    /// SNS-W list id assigned to this deployed SNS.
    pub id: usize,
    /// SNS name resolved during discovery.
    pub name: String,
    /// Root canister queried for inventory and health.
    pub root_canister_id: String,
    /// Root query method used as the inventory authority.
    pub inventory_method: String,
    /// Root ingress method used for operational health.
    pub health_method: String,
    /// Transport kind used for the health call.
    pub health_call_type: SnsCanisterCallType,
    /// Value sent in the Root health request; always false for this read-only report.
    pub health_update_canister_list: bool,
    /// Whether all joined values represent one authoritative point-in-time snapshot.
    pub point_in_time_guaranteed: bool,
    /// Number of canisters in the Root inventory response.
    pub canister_count: usize,
    /// Number of inventory canisters with returned operational status.
    pub health_status_count: usize,
    /// Number of explicit inventory or health relation gaps.
    pub gap_count: usize,
    /// Canonically ordered inventory rows.
    pub canisters: Vec<SnsCanisterRow>,
    /// Canonically ordered typed relation gaps.
    pub gaps: Vec<SnsCanisterGap>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canister_call_type_labels_round_trip() {
        for (call_type, label) in [
            (SnsCanisterCallType::Query, "query"),
            (SnsCanisterCallType::CompositeQuery, "composite_query"),
            (SnsCanisterCallType::IngressUpdate, "ingress_update"),
        ] {
            assert_eq!(
                serde_json::to_string(&call_type).unwrap(),
                format!("\"{label}\"")
            );
            assert_eq!(
                serde_json::from_str::<SnsCanisterCallType>(&format!("\"{label}\"")).unwrap(),
                call_type
            );
            assert_eq!(call_type.as_str(), label);
        }
    }
}
