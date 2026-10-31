use crate::subnet_catalog::{CatalogAssurance, ValidatedSubnetCatalog};
use serde::{Deserialize, Serialize};

///
/// SubnetCatalogReportMetadata
///
/// Collection and policy evidence shared by catalog views and refresh results.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetCatalogReportMetadata {
    /// Canonical Registry canister principal supplying the snapshot.
    pub registry_canister_id: String,
    /// Exact Registry version represented by the catalog.
    pub registry_version: u64,
    /// Assurance established for this exact snapshot.
    pub assurance: CatalogAssurance,
    /// Source endpoints contributing to the snapshot.
    pub source_endpoints: Vec<String>,
    /// Canonical Registry payload digest agreed by every source endpoint.
    pub agreement_digest: Option<String>,
    /// Exact number of Registry query calls made during collection.
    pub registry_query_call_count: u64,
    /// Registry record family selected as routing authority.
    pub routing_source: crate::subnet_catalog::SubnetCatalogRoutingSource,
    /// Per-value Registry provenance retained by the catalog.
    pub registry_records: Vec<crate::subnet_catalog::SubnetCatalogRegistryRecordEvidence>,
    /// Lowercase SHA-256 digest of the canonical catalog payload.
    pub catalog_digest: String,
    /// UTC collection timestamp retained in catalog provenance.
    pub fetched_at: String,
    /// Implementation identity of the recorded routing resolver policy.
    pub resolver_backend: String,
    /// Collector package version recorded by the source.
    pub collector_version: String,
    /// Classification contract version.
    pub classification_schema_version: u32,
    /// Lowercase SHA-256 digest of the classification policy.
    pub classification_policy_digest: String,
    /// Resolver contract version.
    pub resolver_schema_version: u32,
}

impl SubnetCatalogReportMetadata {
    pub(in crate::subnet_catalog) fn from_catalog(catalog: &ValidatedSubnetCatalog) -> Self {
        let provenance = catalog.provenance();
        Self {
            registry_canister_id: provenance.registry_canister_id.clone(),
            registry_version: provenance.registry_version,
            assurance: provenance.assurance,
            source_endpoints: provenance.source_endpoints.clone(),
            agreement_digest: provenance.agreement_digest.clone(),
            registry_query_call_count: provenance.registry_query_call_count,
            routing_source: provenance.routing_source,
            registry_records: provenance.registry_records.clone(),
            catalog_digest: catalog.raw().catalog_digest.clone(),
            fetched_at: provenance.fetched_at.clone(),
            resolver_backend: provenance.resolver_backend.clone(),
            collector_version: provenance.collector_version.clone(),
            classification_schema_version: provenance.classification_schema_version,
            classification_policy_digest: provenance.classification_policy_digest.clone(),
            resolver_schema_version: provenance.resolver_schema_version,
        }
    }
}
