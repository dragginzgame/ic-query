use super::NnsTopologyAssessmentStatus;
use serde::{Deserialize, Serialize};

///
/// NnsTopologyGapsReport
///
/// NNS topology report listing unresolved registry relations.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyGapsReport {
    pub schema_version: u32,
    pub network: String,
    pub source_endpoint: String,
    pub status: NnsTopologyAssessmentStatus,
    pub gap_count: usize,
    pub gaps: Vec<NnsTopologyGapRow>,
}

///
/// NnsTopologyGapRow
///
/// One unresolved subject found while joining topology inputs.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NnsTopologyGapRow {
    pub subject_kind: String,
    pub subject: String,
    pub missing_relation: String,
    pub referenced_id: String,
}
