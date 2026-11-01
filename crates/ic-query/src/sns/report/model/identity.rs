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
/// SnsInstanceContext
///
/// SNS instance identity and acquisition metadata shared by lookup reports.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SnsInstanceContext {
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
}

///
/// SnsReportContext
///
/// SNS identity and collection provenance shared by governance report envelopes.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsReportContext {
    /// Shared record, flattened in JSON.
    #[serde(flatten)]
    pub instance: SnsInstanceContext,
    /// Stable SNS Governance canister identity.
    pub governance_canister_id: String,
}

impl<'de> Deserialize<'de> for SnsReportContext {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // A strict parent must see every consumed key; nested Serde flattening cannot track them.
        #[derive(Deserialize)]
        struct Fields {
            network: String,
            sns_wasm_canister_id: String,
            fetched_at: String,
            source_endpoint: String,
            fetched_by: String,
            id: usize,
            name: String,
            root_canister_id: String,
            governance_canister_id: String,
        }
        let fields = Fields::deserialize(deserializer)?;
        Ok(Self {
            instance: SnsInstanceContext {
                network: fields.network,
                sns_wasm_canister_id: fields.sns_wasm_canister_id,
                fetched_at: fields.fetched_at,
                source_endpoint: fields.source_endpoint,
                fetched_by: fields.fetched_by,
                id: fields.id,
                name: fields.name,
                root_canister_id: fields.root_canister_id,
            },
            governance_canister_id: fields.governance_canister_id,
        })
    }
}
