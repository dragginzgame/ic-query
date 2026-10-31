//! Module: sns::report::model::reports::params
//!
//! Responsibility: SNS governance-parameters report DTO.
//! Does not own: governance calls, parameter conversion, or rendering.
//! Boundary: preserves resolved SNS identity with governance parameters.

use super::governance::SnsGovernanceParameters;
use crate::sns::report::SnsReportContext;
use serde::Serialize;

///
/// SnsParamsReport
///
/// Serializable report for one SNS governance parameter set.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsParamsReport {
    pub schema_version: u32,
    /// SNS identity and provenance, serialized as flat report fields.
    #[serde(flatten)]
    pub context: SnsReportContext,
    pub parameters: SnsGovernanceParameters,
}
