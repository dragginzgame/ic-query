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
    /// Independently collected Governance provider metadata provenance.
    pub governance_source: crate::report::CollectionSourceRequest,
    pub node_providers: Vec<NnsNodeProviderRow>,
}

///
/// NnsNodeProviderRow
///
/// One node provider row projected from governance and registry data.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
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
    /// Independently collected Governance provider metadata provenance.
    pub governance_source: crate::report::CollectionSourceRequest,
    /// Resolved inventory row.
    #[serde(flatten)]
    pub node_provider: NnsNodeProviderRow,
}

///
/// NnsNodeProviderMetadata
///
/// Governance provider metadata collected independently of Registry membership and counts.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NnsNodeProviderMetadata {
    /// Current persisted metadata contract, always 1 before 1.0.
    pub schema_version: u32,
    /// Governance acquisition time, endpoint and collector.
    #[serde(flatten)]
    pub source: crate::report::CollectionSourceRequest,
    /// Governance canister returning the provider list.
    pub governance_canister_id: String,
    /// Provider identities and reward accounts; Registry node counts must be absent.
    pub node_providers: Vec<NnsNodeProviderRow>,
}

#[cfg(feature = "nns-host")]
impl crate::cache_file::JsonCacheReport for NnsNodeProviderMetadata {
    fn schema_version(&self) -> u32 {
        self.schema_version
    }
    fn network(&self) -> &str {
        &self.source.network
    }
}
