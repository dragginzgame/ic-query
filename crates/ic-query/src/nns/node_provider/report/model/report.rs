#[cfg(feature = "nns-host")]
use crate::cache_file::JsonCacheReport;
use serde::{Deserialize, Serialize};

///
/// NnsNodeProviderListReport
///
/// Complete NNS node provider inventory report with source metadata.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeProviderListReport {
    pub schema_version: u32,
    pub network: String,
    pub governance_canister_id: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub node_provider_count: usize,
    pub node_providers: Vec<NnsNodeProviderRow>,
}

#[cfg(feature = "nns-host")]
impl JsonCacheReport for NnsNodeProviderListReport {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }

    fn network(&self) -> &str {
        &self.network
    }
}

#[cfg(feature = "nns-host")]
impl_nns_inventory_report!(
    NnsNodeProviderListReport,
    super::super::NNS_NODE_PROVIDER_LIST_REPORT_SCHEMA_VERSION,
    "node_provider",
    node_provider_count,
    node_providers,
    |report: &NnsNodeProviderListReport| {
        if report.governance_canister_id != crate::ic_registry::MAINNET_GOVERNANCE_CANISTER_ID {
            return Err(format!(
                "governance_canister_id is {}, expected {}",
                report.governance_canister_id,
                crate::ic_registry::MAINNET_GOVERNANCE_CANISTER_ID
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
    pub network: String,
    pub governance_canister_id: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub node_provider_principal: String,
    pub name: Option<String>,
    pub node_count: Option<u32>,
    pub reward_account_hex: Option<String>,
}

///
/// NnsNodeProviderRefreshReport
///
/// Outcome of refreshing the cached NNS node provider inventory.
///

#[cfg(feature = "nns-host")]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsNodeProviderRefreshReport {
    pub schema_version: u32,
    pub network: String,
    pub cache_path: String,
    pub refresh_lock_path: String,
    pub output_path: Option<String>,
    pub governance_canister_id: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub dry_run: bool,
    pub wrote_cache: bool,
    pub replaced_existing_cache: bool,
    pub node_provider_count: usize,
}
