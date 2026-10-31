//! Canonical SNS canister identities shared by discovery, joined catalogs and reports.

use serde::{Deserialize, Serialize};

///
/// MainnetSnsCanisters
///
/// Complete canister identities for one deployed SNS shared by sources and reports.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MainnetSnsCanisters {
    /// SNS Root canister principal.
    pub root_canister_id: String,
    /// SNS Governance canister principal.
    pub governance_canister_id: String,
    /// SNS ledger canister principal.
    pub ledger_canister_id: String,
    /// SNS decentralization-swap canister principal.
    pub swap_canister_id: String,
    /// SNS ledger index canister principal.
    pub index_canister_id: String,
}

///
/// SnsReportContext
///
/// SNS identity and collection provenance shared by governance report envelopes.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SnsReportContext {
    /// Requested IC network identity.
    pub network: String,
    /// SNS-W canister used for discovery.
    pub sns_wasm_canister_id: String,
    /// UTC collection timestamp.
    pub fetched_at: String,
    /// Explicit collection endpoint.
    pub source_endpoint: String,
    /// Collector identity.
    pub fetched_by: String,
    /// Current SNS-W list position.
    pub id: usize,
    /// Current SNS display name.
    pub name: String,
    /// Stable SNS Root canister identity.
    pub root_canister_id: String,
    /// Stable SNS Governance canister identity.
    pub governance_canister_id: String,
}
