//! Module: sns::report::live::types::upgrade
//!
//! Responsibility: SNS Governance and SNS-W upgrade-version Candid wire types.
//! Does not own: transport, validation, report assembly, or rendering.
//! Boundary: mirrors only the bounded native version fields consumed by the live adapter.

use candid::{CandidType, Deserialize, Principal};

///
/// GetRunningSnsVersionRequest
///
/// Empty record accepted by SNS Governance `get_running_sns_version`.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetRunningSnsVersionRequest {}

///
/// SnsVersionWire
///
/// Native SNS version with one Wasm hash per deployed canister role.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
#[expect(
    clippy::struct_field_names,
    reason = "fields mirror the official SNS Candid version record"
)]
pub(in crate::sns::report::live) struct SnsVersionWire {
    pub(in crate::sns::report::live) archive_wasm_hash: Vec<u8>,
    pub(in crate::sns::report::live) root_wasm_hash: Vec<u8>,
    pub(in crate::sns::report::live) swap_wasm_hash: Vec<u8>,
    pub(in crate::sns::report::live) ledger_wasm_hash: Vec<u8>,
    pub(in crate::sns::report::live) governance_wasm_hash: Vec<u8>,
    pub(in crate::sns::report::live) index_wasm_hash: Vec<u8>,
}

///
/// PendingSnsVersion
///
/// Native Governance pending-upgrade state.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct PendingSnsVersion {
    pub(in crate::sns::report::live) mark_failed_at_seconds: u64,
    pub(in crate::sns::report::live) checking_upgrade_lock: u64,
    pub(in crate::sns::report::live) proposal_id: u64,
    pub(in crate::sns::report::live) target_version: Option<SnsVersionWire>,
}

///
/// GetRunningSnsVersionResponse
///
/// Native SNS Governance running-version response.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetRunningSnsVersionResponse {
    pub(in crate::sns::report::live) deployed_version: Option<SnsVersionWire>,
    pub(in crate::sns::report::live) pending_version: Option<PendingSnsVersion>,
}

///
/// GetNextSnsVersionRequest
///
/// Native SNS-W request for the next blessed version after one deployed version.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetNextSnsVersionRequest {
    pub(in crate::sns::report::live) governance_canister_id: Option<Principal>,
    pub(in crate::sns::report::live) current_version: Option<SnsVersionWire>,
}

///
/// GetNextSnsVersionResponse
///
/// Native SNS-W next-version response.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetNextSnsVersionResponse {
    pub(in crate::sns::report::live) next_version: Option<SnsVersionWire>,
}
