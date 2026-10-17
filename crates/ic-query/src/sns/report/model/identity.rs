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
