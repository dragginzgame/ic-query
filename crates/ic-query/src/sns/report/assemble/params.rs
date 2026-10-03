//! Module: sns::report::assemble::params
//!
//! Responsibility: assemble SNS governance parameter reports.
//! Does not own: governance parameter fetching, lookup resolution, or rendering.
//! Boundary: combines resolved SNS identity and raw parameter DTOs into report output.

use crate::sns::report::{
    MAINNET_SNS_WASM_CANISTER_ID, MainnetSns, SNS_PARAMS_REPORT_SCHEMA_VERSION,
    SnsGovernanceParameters, SnsParamsReport, SnsSourceRequest,
};

pub(in crate::sns::report) fn sns_params_report_from_parts(
    fetch_request: SnsSourceRequest,
    sns: MainnetSns,
    parameters: SnsGovernanceParameters,
) -> SnsParamsReport {
    SnsParamsReport {
        schema_version: SNS_PARAMS_REPORT_SCHEMA_VERSION,
        network: fetch_request.network,
        sns_wasm_canister_id: MAINNET_SNS_WASM_CANISTER_ID.to_string(),
        fetched_at: fetch_request.fetched_at,
        source_endpoint: fetch_request.endpoint,
        fetched_by: fetch_request.fetched_by,
        id: sns.id,
        name: sns.name,
        root_canister_id: sns.root_canister_id,
        governance_canister_id: sns.governance_canister_id,
        parameters,
    }
}
