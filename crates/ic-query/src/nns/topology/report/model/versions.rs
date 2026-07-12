use super::NnsTopologyRegistryVersionRow;
use serde::{Deserialize, Serialize};

///
/// NnsTopologyVersionsReport
///
/// NNS topology report comparing component registry versions.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyVersionsReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub source_count: usize,
    pub registry_versions: Vec<NnsTopologyRegistryVersionRow>,
}
