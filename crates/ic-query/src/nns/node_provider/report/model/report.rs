use serde::{Deserialize, Serialize};

///
/// NnsNodeProviderListReport
///
/// Complete NNS node provider inventory report with source metadata.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NnsNodeProviderListReport {
    pub schema_version: u32,
    /// Registry acquisition metadata, flattened in JSON.
    #[serde(flatten)]
    pub provenance: crate::nns::registry::NnsRegistryInventoryProvenance,
    pub governance_canister_id: String,
    pub node_providers: Vec<NnsNodeProviderRow>,
}

#[cfg(feature = "nns-host")]
impl_nns_inventory_report!(
    NnsNodeProviderListReport,
    super::super::NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION,
    "node_provider",
    super::super::COLLECTION,
    node_providers,
    |report: &NnsNodeProviderListReport| Some(report.governance_canister_id.clone()),
    |report: &NnsNodeProviderListReport| {
        if report.governance_canister_id != crate::nns::MAINNET_GOVERNANCE_CANISTER_ID {
            return Err(format!(
                "governance_canister_id is {}, expected {}",
                report.governance_canister_id,
                crate::nns::MAINNET_GOVERNANCE_CANISTER_ID
            ));
        }
        Ok(())
    },
);

///
/// NnsNodeProviderRow
///
/// One node provider row projected from governance and registry data.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeProviderRow {
    pub node_provider_principal: String,
    pub name: Option<String>,
    pub node_count: Option<u32>,
    pub reward_account_hex: Option<String>,
}

///
/// NnsNodeProviderInfoReport
///
/// Detailed report for one resolved NNS node provider.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeProviderInfoReport {
    pub schema_version: u32,
    pub input: String,
    pub resolved_from: String,
    /// Registry acquisition metadata, flattened in JSON.
    #[serde(flatten)]
    pub provenance: crate::nns::registry::NnsRegistryInventoryProvenance,
    pub governance_canister_id: String,
    /// Resolved inventory row.
    #[serde(flatten)]
    pub node_provider: NnsNodeProviderRow,
}
