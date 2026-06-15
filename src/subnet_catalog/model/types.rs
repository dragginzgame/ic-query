use super::{ClassificationSource, GeographicScope, SubnetKind, SubnetSpecialization};
use serde::{Deserialize, Serialize};

///
/// SubnetCatalog
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetCatalog {
    pub catalog_schema_version: u32,
    pub network: String,
    pub registry_canister_id: String,
    pub registry_version: u64,
    pub fetched_at: String,
    pub fetched_by: String,
    pub source_endpoint: String,
    pub resolver_backend: String,
    pub subnets: Vec<SubnetInfo>,
    pub routing_ranges: Vec<RoutingRange>,
}

///
/// SubnetInfo
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubnetInfo {
    pub subnet_principal: String,
    pub subnet_kind: SubnetKind,
    pub subnet_kind_source: ClassificationSource,
    pub subnet_specialization: SubnetSpecialization,
    pub subnet_specialization_source: ClassificationSource,
    pub geographic_scope: GeographicScope,
    pub geographic_scope_source: ClassificationSource,
    pub subnet_label: String,
    pub subnet_label_source: ClassificationSource,
    pub node_count: Option<u32>,
    pub charges_apply_by_default: bool,
}

///
/// RoutingRange
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoutingRange {
    pub start_canister_id: String,
    pub end_canister_id: String,
    pub subnet_principal: String,
}
