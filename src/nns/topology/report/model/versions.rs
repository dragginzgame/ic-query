use super::NnsTopologyRegistryVersionRow;
use serde::{Deserialize, Serialize};

///
/// NnsTopologyVersionsReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyVersionsReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub source_count: usize,
    pub registry_versions: Vec<NnsTopologyRegistryVersionRow>,
}
