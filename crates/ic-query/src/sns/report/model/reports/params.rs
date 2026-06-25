//! Module: sns::report::model::reports::params
//!
//! Responsibility: SNS governance-parameters report DTO.
//! Does not own: governance calls, parameter conversion, or rendering.
//! Boundary: preserves resolved SNS identity with governance parameters.

use super::governance::SnsGovernanceParameters;
use serde::Serialize;

///
/// SnsParamsReport
///
/// Serializable report for one SNS governance parameter set.
///

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SnsParamsReport {
    pub schema_version: u32,
    pub network: String,
    pub sns_wasm_canister_id: String,
    pub fetched_at: String,
    pub source_endpoint: String,
    pub fetched_by: String,
    pub id: usize,
    pub name: String,
    pub root_canister_id: String,
    pub governance_canister_id: String,
    pub parameters: SnsGovernanceParameters,
}
