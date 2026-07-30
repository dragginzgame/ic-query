//! Module: sns::report::live::types::canisters
//!
//! Responsibility: SNS Root inventory and health Candid wire types.
//! Does not own: live transport, report projection, lookup, or rendering.
//! Boundary: mirrors only the Root fields required by the public report.

use candid::{CandidType, Deserialize, Nat, Principal};

///
/// ListSnsCanistersRequest
///
/// Empty Candid request for SNS Root inventory.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct ListSnsCanistersRequest {}

///
/// ListSnsCanistersResponse
///
/// Candid response containing SNS Root's current canister inventory.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct ListSnsCanistersResponse {
    pub(in crate::sns::report::live) root: Option<Principal>,
    pub(in crate::sns::report::live) governance: Option<Principal>,
    pub(in crate::sns::report::live) ledger: Option<Principal>,
    pub(in crate::sns::report::live) swap: Option<Principal>,
    pub(in crate::sns::report::live) index: Option<Principal>,
    pub(in crate::sns::report::live) dapps: Vec<Principal>,
    pub(in crate::sns::report::live) archives: Vec<Principal>,
    pub(in crate::sns::report::live) extensions: Option<SnsRootExtensions>,
}

///
/// SnsRootExtensions
///
/// Extension canister ids nested in the SNS Root inventory response.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct SnsRootExtensions {
    pub(in crate::sns::report::live) extension_canister_ids: Vec<Principal>,
}

///
/// GetSnsCanistersSummaryRequest
///
/// Candid request for SNS Root canister status summaries.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetSnsCanistersSummaryRequest {
    pub(in crate::sns::report::live) update_canister_list: Option<bool>,
}

impl GetSnsCanistersSummaryRequest {
    pub(in crate::sns::report::live) const fn read_only() -> Self {
        Self {
            update_canister_list: Some(false),
        }
    }
}

///
/// GetSnsCanistersSummaryResponse
///
/// Candid response containing status summaries for supported SNS roles.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct GetSnsCanistersSummaryResponse {
    pub(in crate::sns::report::live) root: Option<CanisterSummary>,
    pub(in crate::sns::report::live) governance: Option<CanisterSummary>,
    pub(in crate::sns::report::live) ledger: Option<CanisterSummary>,
    pub(in crate::sns::report::live) swap: Option<CanisterSummary>,
    pub(in crate::sns::report::live) index: Option<CanisterSummary>,
    pub(in crate::sns::report::live) dapps: Vec<CanisterSummary>,
    pub(in crate::sns::report::live) archives: Vec<CanisterSummary>,
}

///
/// CanisterSummary
///
/// Optional canister identity and status returned by SNS Root.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct CanisterSummary {
    pub(in crate::sns::report::live) canister_id: Option<Principal>,
    pub(in crate::sns::report::live) status: Option<CanisterStatusResult>,
}

///
/// CanisterStatusResult
///
/// Operational canister fields retained from the Root status result.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct CanisterStatusResult {
    pub(in crate::sns::report::live) status: CanisterStatusType,
    pub(in crate::sns::report::live) memory_size: Nat,
    pub(in crate::sns::report::live) cycles: Nat,
    pub(in crate::sns::report::live) settings: DefiniteCanisterSettings,
    pub(in crate::sns::report::live) idle_cycles_burned_per_day: Nat,
    pub(in crate::sns::report::live) module_hash: Option<Vec<u8>>,
}

///
/// DefiniteCanisterSettings
///
/// Root status settings subset required by the report.
///

#[derive(CandidType, Clone, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) struct DefiniteCanisterSettings {
    pub(in crate::sns::report::live) controllers: Vec<Principal>,
}

///
/// CanisterStatusType
///
/// Native management-canister running-state variant returned by Root.
///

#[derive(CandidType, Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub(in crate::sns::report::live) enum CanisterStatusType {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopping")]
    Stopping,
    #[serde(rename = "stopped")]
    Stopped,
}

#[cfg(test)]
mod tests {
    use super::GetSnsCanistersSummaryRequest;

    #[test]
    fn health_request_explicitly_disables_inventory_updates() {
        assert_eq!(
            GetSnsCanistersSummaryRequest::read_only().update_canister_list,
            Some(false)
        );
    }
}
