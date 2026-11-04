//! Module: cloud_engine::wire
//!
//! Responsibility: mirror the pinned CloudEngine control-plane Candid interface.
//! Does not own: report provenance, source validation, transport, or rendering.
//! Boundary: changes here follow the ICP CLI 1.3.0 engine-canister contracts exactly.

use super::{CloudEngineNodeType, CloudEnginePriceRow};
use ic_platform::candid::contract::{CandidType, Nat, Principal, serde::Deserialize};

///
/// GetEngineOperatorBySubnetArgs
///
/// Wire argument for `getEngineOperatorBySubnet`.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct GetEngineOperatorBySubnetArgs {
    pub(super) subnet_id: Option<Principal>,
}

///
/// GetEngineOperatorBySubnetResult
///
/// Wire response from `getEngineOperatorBySubnet`.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct GetEngineOperatorBySubnetResult {
    pub(super) engine_operator_id: Option<Principal>,
}

///
/// GetEngineOwnerResult
///
/// Wire response from an engine operator's `getEngineOwner` query.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct GetEngineOwnerResult {
    pub(super) engine_owner: Option<Principal>,
}

///
/// GetPlatformAdminResult
///
/// Wire response from an engine operator's `getPlatformAdmin` query.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct GetPlatformAdminResult {
    pub(super) platform_admin: Option<Principal>,
}

///
/// CaffeineSettings
///
/// Wire Caffeine configuration nested in an operator response.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct CaffeineSettings {
    pub(super) enabled: Option<bool>,
}

///
/// GetCaffeineSettingsResult
///
/// Wire response from an engine operator's `getCaffeineSettings` query.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct GetCaffeineSettingsResult {
    pub(super) settings: Option<CaffeineSettings>,
}

///
/// ListDomainsResult
///
/// Wire response from an engine operator's `listDomains` query.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct ListDomainsResult {
    pub(super) domains: Option<Vec<String>>,
}

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
struct CloudEnginePriceWire {
    gross_cycles: Nat,
    net_cycles: Nat,
}

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
struct CloudEnginePriceEntryWire {
    node_type: CloudEngineNodeType,
    dc: Option<String>,
    provider: Option<Principal>,
    price: CloudEnginePriceWire,
    updated_at: i64,
}

///
/// CloudEngineMarketplaceEntryWire
///
/// Wire row returned by `listMarketplacePrices`.
///

#[derive(Clone, Debug, Eq, PartialEq, CandidType, Deserialize)]
#[candid_path("ic_platform::candid::contract")]
pub(super) struct CloudEngineMarketplaceEntryWire {
    key: String,
    entry: CloudEnginePriceEntryWire,
}

impl CloudEngineMarketplaceEntryWire {
    pub(super) fn into_report_row(self) -> CloudEnginePriceRow {
        CloudEnginePriceRow {
            key: self.key,
            node_type: self.entry.node_type,
            data_center_id: self.entry.dc,
            provider_id: self.entry.provider.map(|principal| principal.to_text()),
            net_cycles_per_month: self.entry.price.net_cycles.0.to_str_radix(10),
            gross_cycles_per_month: self.entry.price.gross_cycles.0.to_str_radix(10),
            updated_at_unix_nanos: self.entry.updated_at,
        }
    }
}

#[cfg(test)]
mod tests;
