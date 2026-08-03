//! Module: sns::report::live::fetch::params
//!
//! Responsibility: fetch SNS governance parameters.
//! Does not own: lookup resolution, report assembly, cache IO, or rendering.
//! Boundary: queries one resolved SNS governance canister for parameters.

use super::{block_on_sns, governance_canister};
use crate::sns::report::{
    SnsGovernanceParameters, SnsHostError,
    live::{
        convert::sns_governance_parameters,
        query::{query_canister, sns_agent},
        types::SnsGovernanceParametersWire,
    },
    source::{MainnetSns, SnsSourceRequest},
};

/// Fetch governance parameters for one resolved mainnet SNS.
pub(in crate::sns::report::live) fn fetch_mainnet_sns_params(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<SnsGovernanceParameters, SnsHostError> {
    block_on_sns(fetch_mainnet_sns_params_async(request, sns))
}

async fn fetch_mainnet_sns_params_async(
    request: &SnsSourceRequest,
    sns: &MainnetSns,
) -> Result<SnsGovernanceParameters, SnsHostError> {
    let agent = sns_agent(request)?;
    let governance_canister = governance_canister(sns)?;
    let parameters: SnsGovernanceParametersWire = query_canister(
        &agent,
        &governance_canister,
        "get_nervous_system_parameters",
        "get_nervous_system_parameters",
        "SnsGovernanceParameters",
        &(),
    )
    .await?;
    Ok(sns_governance_parameters(parameters))
}
