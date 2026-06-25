use serde::{Deserialize, Serialize};

///
/// NnsTopologyGapsReport
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyGapsReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub status: String,
    pub gap_count: usize,
    pub gaps: Vec<NnsTopologyGapRow>,
}

///
/// NnsTopologyGapRow
///
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyGapRow {
    pub subject_kind: String,
    pub subject: String,
    pub missing_relation: String,
    pub referenced_id: String,
}
