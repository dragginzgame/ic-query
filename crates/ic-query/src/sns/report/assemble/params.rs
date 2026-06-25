//! Module: sns::report::assemble::params
//!
//! Responsibility: assemble SNS governance parameter reports.
//! Does not own: governance parameter fetching, lookup resolution, or rendering.
//! Boundary: combines resolved SNS identity and raw parameter DTOs into report output.

use crate::sns::report::{
    MainnetSns, MainnetSnsList, SNS_PARAMS_REPORT_SCHEMA_VERSION, SnsGovernanceParameters,
    SnsParamsReport,
};

pub(in crate::sns::report) fn sns_params_report_from_parts(
    list: MainnetSnsList,
    id: usize,
    sns: MainnetSns,
    parameters: SnsGovernanceParameters,
) -> SnsParamsReport {
    SnsParamsReport {
        schema_version: SNS_PARAMS_REPORT_SCHEMA_VERSION,
        network: list.network,
        sns_wasm_canister_id: list.sns_wasm_canister_id,
        fetched_at: list.fetched_at,
        source_endpoint: list.source_endpoint,
        fetched_by: list.fetched_by,
        id,
        name: sns.name,
        root_canister_id: sns.root_canister_id,
        governance_canister_id: sns.governance_canister_id,
        parameters,
    }
}
